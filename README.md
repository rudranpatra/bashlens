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

  Network       ███████░░░  high
  Privilege     ███░░░░░░░  low
  Persistence   ███░░░░░░░  low
  Obfuscation   ███████░░░  high
  Verification  ❌ none

  Risk percentile: 57th
  Compared against 174 installers in the corpus

[Network]
  [Medium] Performs an outbound HTTP(S) request  evidence: ["curl", "wget", ...]
[Obfuscation]
  [High] Contains obfuscated or dynamic execution primitives  evidence: ["base64 -d"]
```

The bars and percentile are computed by re-scanning this repo's own
`corpus/scripts/` and ranking the target script against it — no network
call, no external service. Run it yourself: `cargo run --bin bashlens --
corpus/scripts/uv` (or point it at any URL).

## The finding

We analyzed 174 real install scripts from tools people actually pipe into
bash — rustup, bun, deno, uv, docker, ollama, homebrew, k3s, tailscale, and
165 others.

**55.7% (97/174) download and execute a binary with no checksum or
signature verification.**

Full methodology, exact metric definitions, and the reproducible collection
pipeline: [`corpus/CORPUS.md`](corpus/CORPUS.md). Full stats:
[`corpus/stats/day0_report.md`](corpus/stats/day0_report.md).

## Why no safe/unsafe verdict

Bash is not statically analyzable in the general case — a sufficiently
motivated script can hide anything from a tool that only reads text. A
verdict would imply a guarantee `bashlens` can't back up, and it would be
false the first time someone bypassed it. What we can do honestly is show
you what the script visibly does and flag what we genuinely can't resolve
(see Limitations below) — that's the whole design.

## What it checks

| Class | What it surfaces |
|---|---|
| **Network** | `curl`/`wget` invocations and the URLs they hit |
| **Privilege** | `sudo` usage |
| **Persistence** | Writes to shell profiles, `profile.d`, systemd units |
| **Verification** | Whether a checksum (`sha256sum`, etc.) or signature (`gpg`, `cosign`, etc.) check is present anywhere in the script |
| **Obfuscation** | `eval`, `base64 -d`, direct pipe-to-shell, process substitution into bash, network-fed `eval $(...)` |

## Install and run

There's no packaged distribution yet (`npx`, Homebrew, prebuilt binaries) —
that's the next milestone, not this one. Today:

```bash
git clone https://github.com/rudranpatra/bashlens.git
cd bashlens
cargo build --release

# Remote script
cargo run --bin bashlens -- https://bun.sh/install

# Local script, markdown output
cargo run --bin bashlens -- corpus/scripts/uv --format markdown
```

The corpus risk-percentile block only appears when run from this repo's root
(it reads `corpus/scripts/` and `rules/` relative to the current directory).
Scanning a script from elsewhere still works — it just skips the percentile
comparison.

## Architecture

```
crates/
  cli/        # clap-based binary (`bashlens`)
  analyzer/   # runs the rule set over a script; scores it against the corpus
  rules/      # YAML rule loader + regex engine
  parser/     # AST parser stub (tree-sitter-bash target — not wired in yet)
  report/     # text / JSON / markdown output, bar-chart rendering
rules/        # YAML detection rules (data, not code)
corpus/       # install-script corpus, metadata, and aggregate stats
day0/         # collection (collect.py) and crude analysis (analyze.py) scripts
```

## Known limitations

- **Regex, not an AST.** v0.1 detection is YAML-defined regex over raw script
  text (`crates/rules`), not a real parse. `tree-sitter-bash` integration is
  planned (`crates/parser` is an intentional stub today) but not shipped.
  This means both false positives (a pattern matching in a comment or a
  string literal) and false negatives (logic split across variables/functions
  that a plain regex won't connect) are possible.
- **Verification is presence, not enforcement.** A `sha256sum` or `gpg` call
  anywhere in the script counts as "verification present" — this does not
  confirm the result is ever checked, or that execution stops on a mismatch.
- **Network detection needs a literal `curl`/`wget`.** A script whose only
  fetching happens through a package manager (`apt-get install`, `brew
  install`) won't be flagged as networked by this metric, even though it is.
- **The corpus percentile is a heuristic**, not a validated risk model: it's
  a severity-weighted count of regex matches, ranked against this repo's own
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
Yes — see Limitations. Static text analysis has a real ceiling. That ceiling
is disclosed, not hidden, because pretending otherwise is worse than the gap
itself.

## Corpus licensing

`corpus/scripts/` contains third-party install scripts, each under whatever
license its own publisher applies — not covered by this repo's MIT license.
See [`corpus/CORPUS.md`](corpus/CORPUS.md#licensing-and-attribution).

## License

MIT — see [`LICENSE`](LICENSE) for the bashlens source itself; the corpus has
its own attribution terms (above).
