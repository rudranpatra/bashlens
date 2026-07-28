//! Thin wrapper around `tree-sitter-bash`. This crate owns the grammar
//! dependency so callers (bashlens-rules, bashlens-analyzer) never touch
//! `tree_sitter` directly - if the grammar version ever changes, this is the
//! only crate that needs to know.

use anyhow::{Context, Result};
pub use tree_sitter::{Node, Query, QueryCursor, Tree};

/// Parses a bash script into a tree-sitter AST. Never fails on malformed
/// input - tree-sitter produces a best-effort tree with ERROR nodes for
/// anything it can't parse, which is exactly what we want for real-world
/// install scripts (many are not strictly POSIX-clean).
pub fn parse(source: &str) -> Result<Tree> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_bash::LANGUAGE.into())
        .context("failed to load tree-sitter-bash grammar")?;
    parser
        .parse(source, None)
        .context("tree-sitter-bash produced no tree")
}

pub fn language() -> tree_sitter::Language {
    tree_sitter_bash::LANGUAGE.into()
}

/// UTF-8 text of a node, tolerant of non-UTF8 source (matches the CLI's
/// lossy-decode policy for makeself-style installers with a binary tail).
pub fn node_text<'a>(node: &Node, source: &'a str) -> &'a str {
    node.utf8_text(source.as_bytes()).unwrap_or("")
}
