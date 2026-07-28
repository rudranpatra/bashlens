use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleFile {
    pub id: String,
    #[serde(rename = "tree_sitter_query", default)]
    pub tree_sitter_query: Option<String>,
    #[serde(default)]
    pub regex: Vec<String>,
    pub severity: String,
    pub description: String,
    pub class: String,
}

pub struct Rule {
    pub id: String,
    pub class: String,
    pub severity: String,
    pub description: String,
    pub regexes: Vec<Regex>,
}

impl Rule {
    pub fn applies(&self, text: &str) -> bool {
        self.regexes.iter().any(|re| re.is_match(text))
    }

    pub fn evidence(&self, text: &str) -> Vec<String> {
        let mut out = Vec::new();
        for re in &self.regexes {
            for mat in re.find_iter(text).take(3) {
                out.push(mat.as_str().to_string());
            }
        }
        out
    }
}

pub struct RuleSet {
    rules: Vec<Rule>,
}

impl RuleSet {
    pub fn load_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let mut rules = Vec::new();

        for entry in std::fs::read_dir(&dir)
            .with_context(|| format!("rules directory {:?} not readable", dir.as_ref()))?
        {
            let entry = entry?;
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "yml" || ext == "yaml" {
                    let content = std::fs::read_to_string(&path)?;
                    let file: RuleFile = serde_yaml::from_str(&content)
                        .with_context(|| format!("failed to parse rule file {:?}", path))?;

                    let mut regexes = Vec::new();
                    for pat in &file.regex {
                        regexes.push(Regex::new(pat).with_context(|| {
                            format!("invalid regex in rule {}: {}", file.id, pat)
                        })?);
                    }

                    rules.push(Rule {
                        id: file.id,
                        class: file.class,
                        severity: file.severity,
                        description: file.description,
                        regexes,
                    });
                }
            }
        }

        Ok(Self { rules })
    }

    pub fn iter(&self) -> impl Iterator<Item = &Rule> {
        self.rules.iter()
    }
}
