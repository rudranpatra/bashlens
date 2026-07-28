//! Loads detection rules from `rules/<id>/rule.yml` + `rules/<id>/query.scm`
//! and evaluates them against a tree-sitter-bash AST.
//!
//! Each rule is a tree-sitter query (structure) plus a small set of literal
//! values from `rule.yml` (what counts as a match) plus a `kind`, which
//! selects the interpreter that knows how to read that query's captures.
//! `kind` exists because a handful of genuinely different match shapes are
//! needed (a bare command name vs. a command paired with a flag vs. two
//! pipeline stages that both need to be present) - the query still does all
//! the structural work of finding candidate nodes; `kind` only decides how
//! the resulting captures are combined into a verdict.

use anyhow::{Context, Result};
use bashlens_parser::{language, node_text};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Query, QueryCursor, Tree};

#[derive(Debug, Deserialize)]
struct RuleFile {
    id: String,
    class: String,
    severity: String,
    description: String,
    kind: String,
    #[serde(default)]
    match_values: Vec<String>,
    #[serde(default)]
    command: Vec<String>,
    #[serde(default)]
    flags: Vec<String>,
    #[serde(default)]
    fetch_commands: Vec<String>,
    #[serde(default)]
    shell_commands: Vec<String>,
    #[serde(default)]
    eval_commands: Vec<String>,
    #[serde(default)]
    target_commands: Vec<String>,
    #[serde(default)]
    match_substrings: Vec<String>,
}

enum RuleKind {
    /// A bare command name, e.g. `sudo`, `eval`, `sha256sum`.
    SimpleCommand,
    /// A command name plus one of its arguments, e.g. `base64 -d`.
    CommandWithFlag,
    /// A network fetch piped directly into a shell: `curl ... | bash`.
    PipelineToShell,
    /// A shell executed over a process-substituted fetch: `bash <(curl ...)`.
    ProcessSubstitutionExec,
    /// `eval` (or similar) over the output of a fetch: `eval "$(curl ...)"`.
    EvalOfFetch,
    /// A file redirect target, e.g. `>> ~/.bashrc`.
    RedirectTarget,
    /// Any word/string content containing an `http(s)://` URL.
    UrlExtraction,
    /// A network command whose target argument is a variable or command
    /// substitution rather than a literal - can't be resolved statically.
    DynamicArgument,
    /// Any word/string content containing one of `match_substrings`, anywhere
    /// except a comment. The broad counterpart to `redirect_target`: many
    /// real installers stage a path in a variable first (`dest=~/.bashrc`)
    /// and redirect to `$dest` later, which `redirect_target` alone misses.
    TextContains,
}

impl RuleKind {
    fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "simple_command" => Self::SimpleCommand,
            "command_with_flag" => Self::CommandWithFlag,
            "pipeline_to_shell" => Self::PipelineToShell,
            "process_substitution_exec" => Self::ProcessSubstitutionExec,
            "eval_of_fetch" => Self::EvalOfFetch,
            "redirect_target" => Self::RedirectTarget,
            "url_extraction" => Self::UrlExtraction,
            "dynamic_argument" => Self::DynamicArgument,
            "text_contains" => Self::TextContains,
            other => anyhow::bail!("unknown rule kind {other:?}"),
        })
    }
}

pub struct Rule {
    pub id: String,
    pub class: String,
    pub severity: String,
    pub description: String,
    kind: RuleKind,
    query: Query,
    config: RuleFile,
}

impl Rule {
    fn capture_index(&self, name: &str) -> Option<u32> {
        self.query
            .capture_names()
            .iter()
            .position(|n| *n == name)
            .map(|i| i as u32)
    }

    /// Runs this rule's query against `tree` and returns human-readable
    /// evidence strings for every match the rule's `kind`-specific logic
    /// accepts. An empty result means the rule did not fire.
    pub fn evaluate(&self, tree: &Tree, source: &str) -> Vec<String> {
        match self.kind {
            RuleKind::SimpleCommand => self.eval_simple_command(tree, source),
            RuleKind::CommandWithFlag => self.eval_command_with_flag(tree, source),
            RuleKind::PipelineToShell => self.eval_pipeline_to_shell(tree, source),
            RuleKind::ProcessSubstitutionExec => self.eval_process_substitution_exec(tree, source),
            RuleKind::EvalOfFetch => self.eval_eval_of_fetch(tree, source),
            RuleKind::RedirectTarget => self.eval_redirect_target(tree, source),
            RuleKind::UrlExtraction => self.eval_url_extraction(tree, source),
            RuleKind::DynamicArgument => self.eval_dynamic_argument(tree, source),
            RuleKind::TextContains => self.eval_text_contains(tree, source),
        }
    }

    /// Word-boundary match rather than exact equality: the query captures
    /// `word`/`raw_string`/`string_content` nodes generally, not only
    /// command-name position, because a lot of real installers stage a
    /// command behind a variable first - e.g. `sh_c='sudo -E sh -c'` then
    /// `$sh_c apt-get install ...` (seen verbatim in Docker's installer), or
    /// `SUDO=("/usr/bin/sudo")` (Homebrew's). Matching only a literal
    /// `command_name` node would miss both, which would be a worse gap than
    /// the false positives this occasionally costs (e.g. a `sudo` mentioned
    /// only in an echoed suggestion to the user) - see README limitations.
    fn eval_simple_command(&self, tree: &Tree, source: &str) -> Vec<String> {
        let Some(cmd_idx) = self.capture_index("cmd") else {
            return Vec::new();
        };
        let needles: Vec<(String, regex::Regex)> = self
            .config
            .match_values
            .iter()
            .filter_map(|v| {
                regex::Regex::new(&format!(r"\b{}\b", regex::escape(v)))
                    .ok()
                    .map(|re| (v.clone(), re))
            })
            .collect();
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.query, tree.root_node(), source.as_bytes());
        let mut out = Vec::new();
        while let Some(m) = matches.next() {
            for cap in m.captures.iter().filter(|c| c.index == cmd_idx) {
                let text = node_text(&cap.node, source);
                if let Some((value, _)) = needles.iter().find(|(_, re)| re.is_match(text)) {
                    out.push(value.clone());
                }
            }
        }
        out
    }

    fn eval_command_with_flag(&self, tree: &Tree, source: &str) -> Vec<String> {
        let (Some(cmd_idx), Some(arg_idx)) = (self.capture_index("cmd"), self.capture_index("arg"))
        else {
            return Vec::new();
        };
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.query, tree.root_node(), source.as_bytes());
        let mut out = Vec::new();
        while let Some(m) = matches.next() {
            let cmd = m
                .captures
                .iter()
                .find(|c| c.index == cmd_idx)
                .map(|c| node_text(&c.node, source));
            let arg = m
                .captures
                .iter()
                .find(|c| c.index == arg_idx)
                .map(|c| node_text(&c.node, source));
            if let (Some(cmd), Some(arg)) = (cmd, arg) {
                if self.config.command.iter().any(|c| c == cmd)
                    && self.config.flags.iter().any(|f| f == arg)
                {
                    out.push(format!("{cmd} {arg}"));
                }
            }
        }
        out
    }

    fn eval_pipeline_to_shell(&self, tree: &Tree, source: &str) -> Vec<String> {
        let (Some(stage_idx), Some(pipe_idx)) =
            (self.capture_index("stage"), self.capture_index("pipe"))
        else {
            return Vec::new();
        };
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.query, tree.root_node(), source.as_bytes());
        let mut groups: HashMap<usize, Vec<String>> = HashMap::new();
        while let Some(m) = matches.next() {
            let stage = m
                .captures
                .iter()
                .find(|c| c.index == stage_idx)
                .map(|c| node_text(&c.node, source).to_string());
            let pipe_id = m
                .captures
                .iter()
                .find(|c| c.index == pipe_idx)
                .map(|c| c.node.id());
            if let (Some(stage), Some(pipe_id)) = (stage, pipe_id) {
                groups.entry(pipe_id).or_default().push(stage);
            }
        }
        groups
            .values()
            .filter(|stages| {
                let has_fetch = stages
                    .iter()
                    .any(|s| self.config.fetch_commands.iter().any(|f| f == s));
                let has_shell = stages
                    .iter()
                    .any(|s| self.config.shell_commands.iter().any(|sh| sh == s));
                has_fetch && has_shell
            })
            .map(|stages| stages.join(" | "))
            .collect()
    }

    fn eval_process_substitution_exec(&self, tree: &Tree, source: &str) -> Vec<String> {
        let (Some(outer_idx), Some(inner_idx)) =
            (self.capture_index("outer"), self.capture_index("inner"))
        else {
            return Vec::new();
        };
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.query, tree.root_node(), source.as_bytes());
        let mut out = Vec::new();
        while let Some(m) = matches.next() {
            let outer = m
                .captures
                .iter()
                .find(|c| c.index == outer_idx)
                .map(|c| node_text(&c.node, source));
            let inner = m
                .captures
                .iter()
                .find(|c| c.index == inner_idx)
                .map(|c| node_text(&c.node, source));
            if let (Some(outer), Some(inner)) = (outer, inner) {
                if self.config.shell_commands.iter().any(|s| s == outer)
                    && self.config.fetch_commands.iter().any(|f| f == inner)
                {
                    out.push(format!("{outer} <({inner} ...)"));
                }
            }
        }
        out
    }

    fn eval_eval_of_fetch(&self, tree: &Tree, source: &str) -> Vec<String> {
        let (Some(outer_idx), Some(inner_idx)) =
            (self.capture_index("outer"), self.capture_index("inner"))
        else {
            return Vec::new();
        };
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.query, tree.root_node(), source.as_bytes());
        let mut out = Vec::new();
        while let Some(m) = matches.next() {
            let outer = m
                .captures
                .iter()
                .find(|c| c.index == outer_idx)
                .map(|c| node_text(&c.node, source));
            let inner = m
                .captures
                .iter()
                .find(|c| c.index == inner_idx)
                .map(|c| node_text(&c.node, source));
            if let (Some(outer), Some(inner)) = (outer, inner) {
                if self.config.eval_commands.iter().any(|e| e == outer)
                    && self.config.fetch_commands.iter().any(|f| f == inner)
                {
                    out.push(format!("{outer} \"$({inner} ...)\""));
                }
            }
        }
        out
    }

    fn eval_redirect_target(&self, tree: &Tree, source: &str) -> Vec<String> {
        let Some(dest_idx) = self.capture_index("dest") else {
            return Vec::new();
        };
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.query, tree.root_node(), source.as_bytes());
        let mut out = Vec::new();
        while let Some(m) = matches.next() {
            if let Some(cap) = m.captures.iter().find(|c| c.index == dest_idx) {
                let text = node_text(&cap.node, source);
                if self
                    .config
                    .match_substrings
                    .iter()
                    .any(|s| text.contains(s.as_str()))
                {
                    out.push(text.to_string());
                }
            }
        }
        out
    }

    fn eval_url_extraction(&self, tree: &Tree, source: &str) -> Vec<String> {
        let Some(w_idx) = self.capture_index("w") else {
            return Vec::new();
        };
        let url_re = regex::Regex::new(r#"https?://[^\s"'<>)]+"#).expect("valid URL regex");
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.query, tree.root_node(), source.as_bytes());
        let mut out = Vec::new();
        while let Some(m) = matches.next() {
            if let Some(cap) = m.captures.iter().find(|c| c.index == w_idx) {
                let text = node_text(&cap.node, source);
                out.extend(url_re.find_iter(text).map(|mat| mat.as_str().to_string()));
            }
        }
        out
    }

    fn eval_dynamic_argument(&self, tree: &Tree, source: &str) -> Vec<String> {
        let Some(cmd_idx) = self.capture_index("cmd") else {
            return Vec::new();
        };
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.query, tree.root_node(), source.as_bytes());
        let mut out = Vec::new();
        while let Some(m) = matches.next() {
            if let Some(cap) = m.captures.iter().find(|c| c.index == cmd_idx) {
                let cmd = node_text(&cap.node, source);
                if self.config.target_commands.iter().any(|t| t == cmd) {
                    out.push(format!(
                        "{cmd}: target built from a variable or command substitution, not a literal"
                    ));
                }
            }
        }
        out
    }

    fn eval_text_contains(&self, tree: &Tree, source: &str) -> Vec<String> {
        let Some(w_idx) = self.capture_index("w") else {
            return Vec::new();
        };
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&self.query, tree.root_node(), source.as_bytes());
        let mut out = Vec::new();
        while let Some(m) = matches.next() {
            if let Some(cap) = m.captures.iter().find(|c| c.index == w_idx) {
                let text = node_text(&cap.node, source);
                if let Some(hit) = self
                    .config
                    .match_substrings
                    .iter()
                    .find(|s| text.contains(s.as_str()))
                {
                    out.push(hit.clone());
                }
            }
        }
        out
    }
}

/// The `rules/` directory embedded into the binary at compile time, so a
/// distributed binary (`cargo install`, Homebrew, `npx`) works from any
/// working directory rather than requiring a checkout of this repo next to
/// it. `RuleSet::embedded()` is what the CLI actually uses; `load_dir` stays
/// available for the fixture test suite and any future `--rules-dir` override.
static EMBEDDED_RULES: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../../rules");

fn build_rule(
    id: &str,
    rule_yaml: &str,
    query_src: &str,
    lang: &tree_sitter::Language,
) -> Result<Rule> {
    let file: RuleFile =
        serde_yaml::from_str(rule_yaml).with_context(|| format!("failed to parse rule {id:?}"))?;
    let query = Query::new(lang, query_src)
        .with_context(|| format!("invalid tree-sitter query in rule {id:?}"))?;
    let kind = RuleKind::parse(&file.kind).with_context(|| format!("in rule {id:?}"))?;
    Ok(Rule {
        id: file.id.clone(),
        class: file.class.clone(),
        severity: file.severity.clone(),
        description: file.description.clone(),
        kind,
        query,
        config: file,
    })
}

pub struct RuleSet {
    rules: Vec<Rule>,
}

impl RuleSet {
    /// Loads every `rules/<id>/{rule.yml,query.scm}` pair under `dir`.
    /// Subdirectories missing either file are skipped rather than erroring,
    /// so a `fixtures/` folder sitting alongside them needs no special-casing.
    pub fn load_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref();
        let lang = language();
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .with_context(|| format!("rules directory {:?} not readable", dir))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        entries.sort_by_key(|e| e.path());

        let mut rules = Vec::new();
        for entry in entries {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let rule_path = path.join("rule.yml");
            let query_path = path.join("query.scm");
            if !rule_path.is_file() || !query_path.is_file() {
                continue;
            }

            let rule_yaml = std::fs::read_to_string(&rule_path)
                .with_context(|| format!("failed to read {:?}", rule_path))?;
            let query_src = std::fs::read_to_string(&query_path)
                .with_context(|| format!("failed to read {:?}", query_path))?;
            let id = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            rules.push(build_rule(&id, &rule_yaml, &query_src, &lang)?);
        }
        Ok(Self { rules })
    }

    /// Same as `load_dir`, reading from the binary's embedded copy of
    /// `rules/` instead of the filesystem. This is what a distributed
    /// binary uses - see `EMBEDDED_RULES`.
    pub fn embedded() -> Result<Self> {
        let lang = language();
        let mut dirs: Vec<_> = EMBEDDED_RULES.dirs().collect();
        dirs.sort_by_key(|d| d.path());

        let mut rules = Vec::new();
        for dir in dirs {
            let Some(rule_file) = dir.get_file(dir.path().join("rule.yml")) else {
                continue;
            };
            let Some(query_file) = dir.get_file(dir.path().join("query.scm")) else {
                continue;
            };
            let id = dir
                .path()
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let rule_yaml = rule_file
                .contents_utf8()
                .with_context(|| format!("embedded rule.yml for {id:?} is not valid UTF-8"))?;
            let query_src = query_file
                .contents_utf8()
                .with_context(|| format!("embedded query.scm for {id:?} is not valid UTF-8"))?;
            rules.push(build_rule(&id, rule_yaml, query_src, &lang)?);
        }
        Ok(Self { rules })
    }

    pub fn iter(&self) -> impl Iterator<Item = &Rule> {
        self.rules.iter()
    }
}
