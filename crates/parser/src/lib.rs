//! Placeholder parser crate.
//!
//! The intended end-state is `tree-sitter-bash` AST integration. For v0.1 / Day 0 the
//! analyzer uses a regex pass. The `tree-sitter` work belongs in this crate later.
use regex::Regex;

/// Crude tokenization. Replaced by an AST walk once the `tree-sitter-bash` grammar is wired in.
pub fn extract_tokens(text: &str) -> Vec<String> {
    text.split_whitespace().map(|s| s.to_string()).collect()
}

/// Extract HTTP(S) URLs for the network-findings pass.
pub fn extract_urls(text: &str) -> Vec<String> {
    let re = Regex::new(r#"https?://[^\s"'<>]+"#).unwrap();
    re.find_iter(text).map(|m| m.as_str().to_string()).collect()
}
