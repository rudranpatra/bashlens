# Changelog

Format loosely follows [Keep a Changelog](https://keepachangelog.com/).

## [0.1.0] - 2026-07-28

Initial release.

### Added

- AST-based install-script analyzer (`tree-sitter-bash`), replacing an
  earlier regex-only pass. 15 detection rules across six behavior classes:
  Network, Privilege, Persistence, Verification, Obfuscation, Unresolvable.
- Research corpus v1: 174 verified real-world install scripts, with a
  reproducible collection pipeline (`day0/collect.py`, `day0/analyze.py`)
  and full methodology (`corpus/CORPUS.md`).
- Headline finding: 55.7% of corpus scripts download and execute a binary
  with no checksum or signature verification.
- Terminal report with per-class risk bars and a corpus risk-percentile,
  computed against the 174-script corpus and cached
  (`corpus/stats/baseline.json`, regenerate via `--update-baseline`).
- `bashlens compare a b c` for side-by-side comparison of several
  installers.
- `--quiet --fail-on <classes>` for CI use.
- `--format text|json|markdown`.
- Rules and the corpus baseline are embedded into the binary at compile
  time, so the compiled binary works from any directory.
- Prebuilt release binaries (Linux x86_64/aarch64 musl, macOS
  x86_64/arm64) via GitHub Actions; npm wrapper and Homebrew formula (not
  yet published to their respective registries/taps).

### Known limitations

See the README's "Known limitations" section - notably, static analysis
cannot see through wrapper-function or staged-variable indirection (a
command executed only via a script-defined function's `"$@"` is invisible
to any static tool), and the corpus risk percentile is a heuristic, not a
validated risk model.
