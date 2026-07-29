# Contributing

## Adding a detection rule

Each rule lives in its own directory under `rules/<id>/`:

```
rules/<id>/
  rule.yml           # id, class, severity, description, kind, and kind-specific config
  query.scm          # the tree-sitter query that finds candidate AST nodes
  fixtures/pass.sh    # a script the rule MUST fire on
  fixtures/fail.sh    # a script the rule must NOT fire on
```

`cargo test --all` runs every rule against its own fixtures automatically
(`crates/rules/tests/fixtures.rs`) - a new rule with fixtures is covered the
moment those two files exist, no test function to add by hand.

See the `kind` values already in use (`simple_command`, `command_with_flag`,
`pipeline_to_shell`, `redirect_target`, `text_contains`, etc. -
`crates/rules/src/lib.rs` has the full list and what each expects in
`rule.yml`) before adding a new one; most detections fit an existing shape.

## Adding a corpus entry

Add `name,url` to `corpus/sources.csv`, then from the repo root:

```bash
python3 day0/collect.py     # fetches everything in sources.csv
python3 day0/analyze.py     # regenerates corpus/stats/day0_*
cargo run --bin bashlens -- --update-baseline   # regenerates the CLI's risk-percentile cache
```

`collect.py` rejects and drops (not just warns on) any response that looks
like an HTML page rather than a script - a URL that stops serving a real
installer becomes an `error` row in `metadata.csv`, not silent corpus
pollution.

## Before opening a PR

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --all
```

All three run in CI (`.github/workflows/ci.yml`) and must pass.

## Reporting a bug in a detection

Open an issue with the script (or a minimal snippet reproducing it) and
what you expected vs. got. If a rule is wrong on a specific real installer,
that installer is exactly the kind of case `rules/<id>/fixtures/` exists to
pin down - a fix PR that adds a fixture reproducing the bug is the fastest
path to landing.

## Security issues

Do not open a public issue for a security vulnerability in `bashlens`
itself (as opposed to a finding *about* a script it analyzes, which is the
whole point of the tool and fine to discuss openly) - see `SECURITY.md`.
