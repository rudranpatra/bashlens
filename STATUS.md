# Status — bashlens v0.1.0

**Last verified:** 2026-07-28. **HEAD:** `09ecdd0`. **Tag:** `v0.1.0`
(GitHub Actions release run succeeded on the second attempt — see
"What actually failed" below).

This document exists to be handed to an independent reviewer (human or
another AI) without requiring them to reconstruct context from chat
history. Every claim below states how it was verified, not just that it
was. Where something was not verified, that's stated plainly instead of
implied.

## TL;DR

Day 0 research gate: passed, reproducible. Engineering (AST rule engine,
CLI, release, npm/Homebrew plumbing): built and verified against real
infrastructure, including one real bug found and fixed by actually running
it. Two items are deliberately **not** done, by explicit user decision or
genuine capability limits — see "Explicitly not done" — not oversights.

## 1. Day 0 research gate

- **Corpus v1:** 174 real installer scripts, collected from `corpus/sources.csv`
  via `day0/collect.py`. Full methodology, collection date, and licensing
  notes: `corpus/CORPUS.md`.
- **Headline finding:** 55.7% (97/174) download and execute a binary with
  no checksum/signature verification. Computed by `day0/analyze.py`, a
  deliberately crude regex pass independent of the Rust CLI (so the
  headline number doesn't depend on the engine being audited alongside it).
- **Reproducibility:** re-ran `python3 day0/analyze.py` live during this
  session; output matched the number in `README.md` and
  `corpus/stats/day0_report.md` exactly.
- **Gate decision:** GO (≥30% threshold from the plan's own Section 6),
  recorded in `corpus/stats/day0_report.md`.

## 2. Engineering

- **Detection engine:** real `tree-sitter-bash` AST (`crates/parser`), not
  regex. 15 rules, each a tree-sitter query (`rules/<id>/query.scm`) +
  metadata (`rule.yml`) + a passing/failing fixture, all checked by
  `cargo test --all` (`crates/rules/tests/fixtures.rs`) — 0 tests existed
  in this repo before this work.
- **Concrete, provable improvements over an earlier regex pass** (found by
  diffing behavior, not assumed): comments are structurally excluded (a
  regex engine flagged a script for "piping curl to bash" based on a
  comment showing users the install command, not real code); an unrelated
  PowerShell filename substring no longer trips the bash profile-write
  rule.
- **Known, disclosed trade-off:** command/string matching was deliberately
  widened to catch the common `sh_c='sudo -E sh -c'` staged-variable
  privilege pattern (verbatim in Docker's real installer) that a
  literal-command-name-only match would miss — documented in
  `README.md` → Known limitations, not hidden.
- **Portability:** rules and the corpus risk-percentile baseline are
  embedded into the binary at compile time (`include_dir!`/`include_str!`).
  Verified by running the compiled binary from an empty `/tmp` directory
  with no `rules/` or `corpus/` present — this was a real bug before the
  fix (the binary hard-required a `rules/` directory in the current
  working directory, which would have broken every distribution channel).
- **Performance:** corpus risk-percentile computation is cached
  (`corpus/stats/baseline.json`, regenerate via `--update-baseline`) rather
  than re-parsing all 174 scripts per invocation — measured ~3.5s → ~140ms.
- **CLI surface:** `bashlens <url|path>`, `bashlens compare a b c`,
  `--format json|markdown|text`, `--quiet --fail-on <classes>` — all
  exercised in this session against real URLs and real corpus scripts.
- **CI:** `.github/workflows/ci.yml` runs fmt/clippy(-D warnings)/test/build
  on every push; green on `705654f`, confirmed via the GitHub Actions API
  at time of writing.

## 3. Release (v0.1.0)

- **What actually failed, and was fixed:** the first `v0.1.0` tag
  triggered a real build on GitHub's infrastructure. 3 of 4 targets
  succeeded; `aarch64-unknown-linux-musl` failed because the workflow used
  `gcc-aarch64-linux-gnu` (a **glibc** cross-compiler) to link a **musl**
  target — a genuine toolchain mismatch, not a hypothetical risk. Fixed by
  switching both Linux legs to build inside `cross-rs/cross`'s Docker
  images. Old tag deleted, re-tagged on the fix commit, second run
  succeeded. Commit `5777f72`.
- **Verified against the real release**, not assumed:
  - Downloaded all 4 tarballs + `checksums.txt` from
    `github.com/rudranpatra/bashlens/releases/tag/v0.1.0`.
  - `sha256sum -c checksums.txt` passed for all 4.
  - Confirmed file format of all 4 binaries: ELF x86_64 and ELF aarch64
    (Linux), Mach-O x86_64 and Mach-O arm64 (macOS).
  - **Executed** the Linux x86_64 binary from a clean `/tmp` with a real
    network fetch (`bashlens https://sh.rustup.rs`) — output matched the
    README's example byte-for-byte.
  - Did **not** execute the macOS or aarch64-Linux binaries — this
    environment has no macOS or ARM hardware. File-format correctness is
    verified; runtime behavior on those platforms is not.

## 4. Packaging

- **npm (`npm/`):** ran the real `install.js` against the real v0.1.0
  release (download, extract, chmod) and the real `bin/bashlens.js`
  against the result, including `compare`. **Not published to the npm
  registry** — that requires npm credentials this session doesn't have and
  won't fabricate. `npx bashlens` will not resolve until that publish
  happens.
- **Homebrew (`homebrew/bashlens.rb`):** real sha256 checksums from the
  actual release are filled in (previously placeholders); all 4 download
  URLs confirmed reachable (HTTP 200). **`brew install` itself was not
  run** — no Homebrew/macOS in this environment. Also not yet published to
  a tap (e.g. `homebrew-bashlens`), which `brew install
  rudranpatra/bashlens/bashlens` would require.

## 5. Documentation / credibility

- Every command in README's "Install and run" section was actually run
  during this session, not copy-checked by eye.
- Grepped the full repo for overclaim language (`detects`, `guarantees`,
  `proves`, `safe`, `secure`, `malicious`): every hit is a correct
  disclaimer ("it never tells you a script is safe") or narrow factual
  usage — no overclaims found.
- `LICENSE` (MIT, scoped to the bashlens source) and a licensing/attribution
  note for the third-party corpus scripts (`corpus/CORPUS.md` →
  "Licensing and attribution") are both in place, with an explicit "this
  is not a legal opinion" caveat rather than a confident-sounding guess.
- **55.7% re-verified a second time**, this session, via full delete +
  regenerate of `corpus/stats/day0_stats.json` and `day0_report.md` (not
  just re-reading the existing file) — byte-identical to the committed
  version.
- **Real screenshots, not mockups:** `docs/img/rustup.svg` and
  `docs/img/compare.svg`, generated with `termtosvg` by actually running
  the `v0.1.0` release binary (single-scan and `compare` against
  bun/deno/uv live over the network), embedded in the README. No PNG
  conversion was done (no SVG-to-PNG renderer available in this
  environment) — noted as an open item in `launch.md` for social platforms
  that prefer raster images.
- **Repo hygiene added:** `CONTRIBUTING.md`, `SECURITY.md`, `CHANGELOG.md`,
  `.github/dependabot.yml` (cargo/npm/github-actions ecosystems). Skipped
  `CODE_OF_CONDUCT.md` — optional per the review this responds to, and
  deliberately not added for a solo v0.1 project.
- **Code-quality scan:** zero `TODO`/`FIXME`/`dbg!()` anywhere in the repo;
  the only `panic!()` is in the test suite's own failure path (correct,
  not a smell); **zero `.unwrap()` calls in any library/binary source
  file** — the one `.expect()` (`crates/rules/src/lib.rs`) is on a
  hardcoded, always-valid regex literal, not on any adversarial input path.
- **Secret scan:** `gitleaks`/`trufflehog` aren't installed in this
  environment (no sudo); ran a manual regex sweep for common secret
  patterns (AWS keys, private-key headers, GitHub/Slack/Google/OpenAI-style
  tokens, generic `api_key=` assignments) across every tracked file, plus a
  filename check for `.env`/`.pem`/`.key`/`id_rsa`/credential files — zero
  hits either way.

## 6. Explicitly not done

Stated plainly, not hedged:

- **Private review (5–10 people, "what's the first thing you don't
  believe?").** Recommended by the plan and by an external review pass;
  **explicitly declined by the user** in this session in favor of
  launching now. This is a real, acknowledged gap in validation, not an
  oversight — nobody outside this conversation has tried to poke a hole in
  the 55.7% methodology or the AST engine's blind spots yet.
- **`vet-run/vet` maintainer outreach.** The plan's Section 7 asks for a
  10-minute issue on `vet-run/vet` asking whether the maintainer intends to
  return, before using the "quiet for 11 months" framing in launch copy.
  Draft text is in `launch.md`; whether it's been posted is not verifiable
  from this repo.
- **Launch posts not yet published.** Drafted in `launch.md` (HN, Lobsters,
  r/programming, X, LinkedIn), following the plan's "lead with the finding,
  name `vet`, no safe/unsafe language" rules. Not posted anywhere — that's
  a user action.
- **GitHub repo settings requiring the owner's own auth** (Discussions,
  Dependabot alerts toggle, repository social-preview image upload): not
  something this session can do without repo-admin credentials. The
  `dependabot.yml` config file is in place; the *setting* that enables
  Dependabot alerts on the repo still needs to be flipped in GitHub's UI.
- **`brew install` / npm registry publish** — see Packaging above.
- **macOS and aarch64-Linux binary execution** — see Release above; this
  environment has neither platform.

## How to re-verify any of this

Every check above is reproducible without trusting this document:

```bash
python3 day0/analyze.py                              # headline number
cargo test --all                                      # 15 rule fixtures
cargo build --release && ./target/release/bashlens --update-baseline
curl -sL -o c.txt https://github.com/rudranpatra/bashlens/releases/download/v0.1.0/checksums.txt
curl -sL -o b.tar.gz https://github.com/rudranpatra/bashlens/releases/download/v0.1.0/bashlens-x86_64-unknown-linux-musl.tar.gz
sha256sum -c c.txt --ignore-missing
```
