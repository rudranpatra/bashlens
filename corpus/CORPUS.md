# Corpus v1

**Frozen:** 2026-07-28
**Size:** 174 verified installer scripts
**Gate:** GO — see `stats/day0_report.md`

Per the project brief's own instruction ("don't keep chasing 500 — freeze the
dataset that supports v0.1"), this is the dataset behind the v0.1 findings and
rule validation. Future re-runs are new versions (`Corpus v2`, etc.), not
silent edits to this one — see "Versioning" below.

## What's in it

- `sources.csv` — curated `name,url` list. This is the only input a human
  edits by hand.
- `scripts/` — the fetched script bodies, one file per source, named after
  the `name` column.
- `metadata.csv` — one row per source: fetch status, byte size, SHA-256 of
  the stored (possibly truncated — see below) content, and any error/note.
- `stats/day0_stats.json` and `stats/day0_report.md` — aggregate counts and
  percentages produced by `day0/analyze.py`.

## Reproducing it

```
python3 day0/collect.py   # fetches sources.csv -> scripts/ + metadata.csv
python3 day0/analyze.py   # scripts/ + metadata.csv -> stats/
```

Both scripts are idempotent and safe to re-run; `collect.py` overwrites
`scripts/` and `metadata.csv` from `sources.csv` each time.

## Collection integrity notes (read before citing this data)

A few real issues turned up while assembling this corpus, and the fixes are
worth stating explicitly rather than leaving implicit:

- **HTML pages are rejected, not stored.** Several vanity install URLs
  (`clickhouse.com/`, `tea.xyz`, `pkgx.sh`, and `charm.sh/install.sh` as of
  this collection) now serve a marketing page instead of a script to a
  plain-curl request. `day0/collect.py` detects this (checks for an
  `<!doctype html>`/`<html` prefix) and records it as an `error` row rather
  than silently storing a webpage as if it were shell source.
- **Makeself-style installers are truncated to 256 KB.** Miniconda and
  Mambaforge ship a small shell header followed by a multi-hundred-megabyte
  binary payload appended to the same file (Miniconda's uncapped download was
  197 MB; Mambaforge's was 105 MB). Only the header is ever shell code, so
  anything past `MAX_SCRIPT_BYTES` (256 KB — comfortably larger than any real
  shell header we've seen, including these two) is discarded before the file
  is written or hashed. This isn't just a size optimization: the discarded
  binary payloads were producing spurious pattern matches (stray ASCII
  substrings inside the binary noise coincidentally matching signature/hex
  regexes), which is why the percentages shifted slightly between the
  untruncated and truncated runs. `metadata.csv` notes which rows were
  truncated and from what original size.
- **Every URL in `sources.csv` is a URL that has actually returned a real
  script**, not a guessed path. An earlier pass included several
  `raw.githubusercontent.com/.../install.sh` guesses that 404'd (the repo
  never hosted a script at that path); those were removed rather than
  "fixed" with another guess.

## Methodology — what each metric actually measures

`day0/analyze.py` is deliberately crude: regex over raw script text, no
parsing, no execution. That's a feature for Day 0 (fast, auditable, no
tree-sitter dependency) but it means every metric below is a **text-pattern
presence check**, not a behavioral guarantee. Before citing any number
publicly, know exactly what it does and doesn't claim:

| Metric | Regex checks for | What it does NOT tell you |
|---|---|---|
| `sudo` | the literal token `sudo` | Whether it's used once or fifty times; whether the elevated command is destructive |
| `network` | `curl` or `wget` as a word | **Package-manager-only installs are undercounted.** A script whose only network activity is `apt-get install`/`brew install` won't match this and is counted as non-network, even though it fetches over the network via the package manager |
| `checksum` | `sha256sum`, `shasum`, `sha512sum`, `sha1sum`, `md5sum`, `cksum` | Whether the computed hash is ever *compared* against a known-good value, or just printed. A script that prints a hash for the user to eyeball manually counts the same as one that fails closed on mismatch |
| `signature` | `gpg`, `gpgv`, `gpg2`, `gpgv2`, `cosign`, `minisign` | Same caveat as checksum — presence, not enforcement |
| `piped_to_shell` | `\| sh` or `\| bash` | Only catches the *direct* pipe idiom. `curl -o x.sh && chmod +x x.sh && ./x.sh` (fetch-then-execute) is not "piped to shell" in this metric's narrow sense, even though it's the same trust exposure |
| `profile` | writes referencing `.bashrc`, `.zshrc`, `.profile`, `profile.d`, fish config | — |
| `systemd` | `systemctl`, `/etc/systemd`, `/usr/lib/systemd` | — |
| `eval` | the literal token `eval` | Any use of `eval`, not just ones evaluating network-fetched content |
| `base64_decode` | `base64 -d` / `base64 --decode` | — |
| `xxd` | `xxd` used anywhere | — |

**The headline metric — "network without verification"** — is: `network`
is true AND neither `checksum` nor `signature` is true, for that script.
Precise answers to the questions this will get asked:

- **Does it distinguish "downloads a binary" from "downloads and immediately
  executes it"?** No. `network` fires on any `curl`/`wget` invocation
  regardless of what happens to the result.
- **Does TLS count as verification?** No. Nearly every URL in this corpus is
  `https://`; transport encryption is not treated as artifact verification.
  "Verification" here means an integrity/authenticity check of the
  downloaded *content* (hash or signature), independent of the transport
  it arrived over.
- **Does GPG count? Cosign?** Yes to both — see the `signature` regex above.
- **What about package-manager installs (apt/brew/etc.)?** Not distinguished
  from raw binary downloads, and in fact *undercounted* as "network" at all
  (see the `network` row above) since they typically don't invoke
  `curl`/`wget` directly. This corpus does not currently give
  package-manager-mediated installs credit for the verification chain their
  distro/registry already provides.

This table is the answer to "define that metric precisely" before the number
appears in any README or launch post — copy the relevant rows rather than
re-deriving them.

## Licensing and attribution

Each file in `scripts/` is the verbatim, unmodified install script published
by that project at the URL recorded in `sources.csv`/`metadata.csv` — it
remains the property of its original publisher under whatever license or
terms that publisher applies (most are MIT/Apache/BSD OSS projects; a few
are install scripts for proprietary CLIs, e.g. cloud-provider tools, where
the script itself is typically publicly redistributable but the underlying
product is not open source). This repository does not claim any license or
ownership over the corpus content, redistributes it unmodified for security
research and transparency purposes, and attributes every script to its
source URL. **This is not a legal opinion** — if you plan to redistribute
this corpus commercially, or a publisher raises an objection, treat it as a
per-source question and consult someone qualified rather than this note.
If a publisher objects to inclusion, remove that entry from `sources.csv`
and `scripts/` on request.

## Versioning

This is Corpus v1. A future re-run with a materially larger or refreshed
source list should be recorded as v2 (new date, new size, new stats snapshot)
rather than overwriting this document's numbers in place — that's what makes
"installer security, N months later" a real trend comparison instead of an
implicit revision nobody can audit.
