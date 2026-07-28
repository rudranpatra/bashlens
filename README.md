# bashlens

Inspect shell install scripts before you pipe them into your shell.

```
cargo run --bin bashlens -- https://example.com/install.sh
```

## Architecture

```
crates/
  cli/        # clap-based binary (`bashlens`)
  analyzer/   # runs the rule set over a script
  rules/      # YAML rule loader + regex engine
  parser/     # AST parser stub (tree-sitter-bash target)
  report/     # text / JSON / markdown output types
rules/        # YAML detection rules (data, not code)
corpus/       # install scripts, metadata, and statistics
```

## Build

```bash
cargo build --release
```

## Run

```bash
# Remote script
cargo run --bin bashlens -- https://bun.sh/install

# Local script
cargo run --bin bashlens -- /path/to/install.sh --format markdown
```

## v0.1 scope

- Static rule-based analysis using YAML rules and regex.
- Three output formats: `text`, `json`, `markdown`.
- `tree-sitter-bash` integration is the next parser milestone; the crate is intentionally a stub today.

## Day 0 / Corpus

The `corpus/` directory is for the install-script research corpus. Scripts, metadata JSON, and aggregate stats live there. The CLI does not require a database.
