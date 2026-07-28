# bashlens

> You wouldn't run a random binary. So why do we run random install scripts?

`bashlens` reads a shell install script and reports what it will do to your
machine — network calls, privilege escalation, persistence, obfuscated
execution, and whether the thing it downloads is ever actually verified —
before you pipe it into your shell.

**It never tells you a script is safe.** Static analysis of bash cannot prove
that, and a tool that claims otherwise is lying the first time someone
bypasses it. It describes what it observes and says plainly what it can't
resolve.

## Real output, not a mockup

This is the actual output of this repository's own binary, run against
`rustup`'s real install script, from the corpus committed in this repo
(`corpus/scripts/rustup`):

```
$ bashlens corpus/scripts/rustup

corpus/scripts/rustup
sha256: 6c30b75a75b28a96fd913a037c8581b580080b6ee9b8169a3c0feb1af7fe8caf

  Network       █████░░░░░  medium
  Privilege     ███░░░░░░░  low
  Persistence   ███░░░░░░░  low
  Obfuscation   ████░░░░░░  medium
  Verification  ❌ none

  Risk percentile: 44th
  Compared against 174 installers in the corpus

[Network]
  [Medium] Performs an outbound HTTP(S) request  evidence: ["curl", "curl", "wget", ...]
  [Medium] References an HTTP(S) URL  evidence: ["https://static.rust-lang.org/rustup", ...]
[Unresolvable]
  [Low] A network command's target is built from a variable or command
  substitution rather than a literal - cannot be resolved statically
  evidence: ["curl: target built from a variable or command substitution, not a literal", ...]
```

The bars and percentile are computed by ranking the target script against
this repo's own 174-script corpus — no network call, no external service.
Run it yourself: `cargo run --bin bashlens -- corpus/scripts/uv` (or point it
at any URL). Compare several side by side:

```
$ bashlens compare corpus/scripts/bun corpus/scripts/uv corpus/scripts/docker

                       Verification   Network   Persistence Obfuscation
corpus/scripts/bun     no             7th       87th        41st
corpus/scripts/uv      yes            62nd      80th        41st
corpus/scripts/docker  yes            51st      77th        41st
```

## The finding

We analyzed 174 real install scripts from tools people actually pipe into
bash — rustup, bun, deno, uv, docker, ollama, homebrew, k3s, tailscale, and
165 others.

**55.7% (97/174) download and execute a binary with no checksum or
signature verification.**

Full methodology, exact metric definitions, and the reproducible collection
pipeline: [`corpus/CORPUS.md`](corpus/CORPUS.md). Full stats:
[`corpus/stats/day0_report.md`](corpus/stats/day0_report.md). Note that this
headline number comes from a separate, deliberately crude regex pass
(`day0/analyze.py`) kept independent of the CLI below — see CORPUS.md for why.

## Why no safe/unsafe verdict

Bash is not statically analyzable in the general case — a sufficiently
motivated script can hide anything from a tool that only reads text. A
verdict would imply a guarantee `bashlens` can't back up, and it would be
false the first time someone bypassed it. What we can do honestly is show
you what the script visibly does and flag what we genuinely can't resolve
(see Limitations below) — that's the whole design.

## What it checks

Detection runs against a real `tree-sitter-bash` AST, not raw-text regex —
each rule is a tree-sitter query (`rules/<id>/query.scm`) plus metadata
(`rule.yml`) plus a passing/failing fixture, and `cargo test` checks every
rule against its own fixtures on every build.

| Class | What it surfaces |
|---|---|
| **Network** | `curl`/`wget` invocations, the URLs they hit, and outbound POST/data-upload flags (`-d`, `--data`, `-X`, `-F`) |
| **Privilege** | `sudo` usage, including the common `SUDO="sudo"; $SUDO apt-get ...` indirection idiom |
| **Persistence** | Writes to shell profiles (`.bashrc`, `.zshrc`, `profile.d`, ...) or `systemctl` service installs |
| **Verification** | Whether a checksum (`sha256sum`, etc.) or signature (`gpg`, `cosign`, etc.) check is present anywhere in the script |
| **Obfuscation** | `eval`, `base64 -d`, `xxd -r`, a fetch piped directly into a shell, a shell run over a process-substituted fetch, `eval` over a fetch's output |
| **Unresolvable** | A network command's target built from a variable or command substitution rather than a literal - reported explicitly rather than silently skipped |

## Install and run

```bash
git clone https://github.com/rudranpatra/bashlens.git
cd bashlens
cargo build --release

# Remote script
cargo run --bin bashlens -- https://bun.sh/install

# Local script, markdown output
cargo run --bin bashlens -- corpus/scripts/uv --format markdown

# CI use: silent unless a forbidden class is present, exit 1 if so
cargo run --bin bashlens -- --quiet --fail-on obfuscation,unresolvable https://example.com/install.sh
```

Rules and the corpus risk-percentile baseline are **embedded into the binary
at compile time** (see Architecture), so the compiled binary works from any
directory — it does not need a checkout of this repo sitting next to it
(verified: the released binary runs correctly from an empty `/tmp` directory).

**Prebuilt binaries:** [GitHub Releases](https://github.com/rudranpatra/bashlens/releases) -
built via `.github/workflows/release.yml`, downloaded and run for real as
part of verifying this release (checksums matched, binary executed
correctly on a clean machine with no repo checkout).

```bash
curl -fsSL -o bashlens.tar.gz \
  https://github.com/rudranpatra/bashlens/releases/download/v0.1.0/bashlens-x86_64-unknown-linux-musl.tar.gz
tar xzf bashlens.tar.gz && ./bashlens https://bun.sh/install
```

(swap `x86_64-unknown-linux-musl` for `aarch64-unknown-linux-musl`,
`x86_64-apple-darwin`, or `aarch64-apple-darwin` as needed.)

**`npx`:** the wrapper in `npm/` downloads the release asset above and was
tested against it end-to-end (download, extract, execute, including
`compare`) - not yet published to the npm registry, so `npx bashlens` won't
resolve until that publish happens.

**Homebrew:** `homebrew/bashlens.rb` has the real checksums from this
release filled in and its URLs verified reachable, but isn't published to a
tap yet, so `brew install` from it wasn't run end-to-end.

`cargo install --path crates/cli` from a clone also works.

## Architecture

```
crates/
  cli/        # clap-based binary (`bashlens`) - compare, --quiet, --fail-on
  analyzer/   # runs the rule set over a script's AST; scores it against the corpus
  rules/      # tree-sitter query engine - loads rules/<id>/{rule.yml,query.scm}
  parser/     # tree-sitter-bash wrapper (real AST, not a stub)
  report/     # text / JSON / markdown output, bar-chart rendering
rules/        # one directory per detection: rule.yml + query.scm + fixtures/{pass,fail}.sh
corpus/       # install-script corpus, metadata, and aggregate stats
day0/         # collection (collect.py) and crude analysis (analyze.py) scripts
npm/          # npx wrapper (downloads the release binary on install)
homebrew/     # Homebrew formula, real checksums, not yet published to a tap
```

Rules and the corpus baseline are compiled into the binary via `include_dir!`/
`include_str!`. After changing `rules/` or `corpus/scripts/`, run
`bashlens --update-baseline` (from the repo root) to regenerate
`corpus/stats/baseline.json` before rebuilding, or percentiles will silently
reflect the old rules.

## Known limitations

- **Static analysis can't see through indirection.** Bash lets a script stage
  a command behind a variable or wrapper function first - e.g. Docker's own
  installer does `sh_c='sudo -E sh -c'` then invokes `$sh_c` later, and both
  Rustup and uv wrap risky calls in a local `ignore()`/similar helper that
  forwards its arguments. `bashlens` widens matching to catch the common
  staged-variable cases (see `rules/sudo`, `rules/profile-mention`), but a
  command executed only through a wrapper function's `"$@"` is invisible to
  any static tool, ours included - there's no way to know a script-defined
  function re-executes its arguments without evaluating it.
- **That same widening trades some precision for recall.** Because
  `sudo`/profile-path matching also looks inside strings (not just literal
  command names), a script that only *mentions* `sudo` in a printed
  suggestion to the user (rather than running it) can be flagged too. We'd
  rather over-report a mention than silently miss the very common
  variable-staged pattern above - but treat a single low-evidence match with
  appropriate skepticism.
- **Verification is presence, not enforcement.** A `sha256sum` or `gpg` call
  anywhere in the script counts as "verification present" — this does not
  confirm the result is ever checked, or that execution stops on a mismatch.
- **Network detection needs a literal `curl`/`wget`.** A script whose only
  fetching happens through a package manager (`apt-get install`, `brew
  install`) won't be flagged as networked by this metric, even though it is.
- **The corpus percentile is a heuristic**, not a validated risk model: it's
  a severity-weighted count of AST matches, ranked against this repo's own
  174-script corpus. Read it as "more or less of this behavior than most
  installers we've seen," not a calibrated probability of harm.
- **Static analysis is defeatable.** A script can construct URLs, commands,
  or entire second-stage payloads dynamically in ways no text-level tool can
  resolve. `bashlens` reports what it can't parse rather than guessing at it
  — but it cannot promise completeness against an adversarial script.

## FAQ

**Why not just read the script yourself?**
You should, when you can — but most install scripts are 100-500 lines with
function indirection, and in practice almost nobody actually reads them
before piping to `bash`. `bashlens` compresses that read into a few seconds
of structured output. It's a faster first pass, not a replacement for
judgment on anything it flags.

**Isn't this the same idea as `vet`?**
Yes, and it should be credited as such — [`vet-run/vet`](https://github.com/vet-run/vet)
proved this category has real demand (1,000+ stars) and has had no commits
in months. `bashlens` exists because the demand `vet` proved didn't go away
when the project did.

**Can a script bypass this?**
Yes — see Limitations. Static analysis, AST-based or not, has a real
ceiling. That ceiling is disclosed, not hidden, because pretending otherwise
is worse than the gap itself.

## Corpus licensing

`corpus/scripts/` contains third-party install scripts, each under whatever
license its own publisher applies — not covered by this repo's MIT license.
See [`corpus/CORPUS.md`](corpus/CORPUS.md#licensing-and-attribution).

## License

MIT — see [`LICENSE`](LICENSE) for the bashlens source itself; the corpus has
its own attribution terms (above).
