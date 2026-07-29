# Launch copy

Drafted per `implementation_plan.md` Section 7 ("Launch") — lead with the
finding, not the tool; name `vet` explicitly; no safe/unsafe language
anywhere. All numbers below are reproducible via `python3 day0/analyze.py`
and were last verified live on 2026-07-28 (see `STATUS.md`).

**Before posting any of these:** open a 10-minute issue on `vet-run/vet`
asking whether the maintainer intends to return (plan Section 7). The
"quiet for 11 months" framing below leans on that.

## Sequence

Per the plan: GitHub Release (done, v0.1.0) → private review (skipped, by
explicit choice — see `STATUS.md`) → Lobsters → Hacker News → Reddit →
LinkedIn → X.

---

## Hacker News

**Title** (submit Tue–Thu, ~8am ET):
> 174 install scripts analyzed: 55.7% verify nothing before executing

**Submission:** link to `https://github.com/rudranpatra/bashlens` (the repo
is the methods section, not a separate write-up). Post this as the first
comment immediately after submitting:

> I analyzed 174 real-world install scripts — the `curl | bash` kind that
> ship with tools like rustup, docker, homebrew, ollama, k3s, tailscale.
> 55.7% (97/174) download and execute a binary with no checksum or
> signature check anywhere in the script.
>
> `vet` (github.com/vet-run/vet) proved this category has real demand —
> 1,000+ stars in about two months — then went quiet for 11 months with no
> successor. This started as picking that back up, and turned into
> `bashlens`: an AST-based (tree-sitter-bash) static analyzer plus the
> corpus behind the numbers.
>
> It doesn't tell you a script is safe — static analysis can't prove that,
> and I didn't want to ship a claim I'd have to walk back the first time
> someone bypassed it. It reports what's observable (network calls,
> privilege escalation, persistence, obfuscated execution) and says
> plainly what it can't resolve (e.g. a URL built from a variable rather
> than a literal).
>
> Corpus, methodology, and full findings: `corpus/CORPUS.md` and
> `corpus/stats/day0_report.md` in the repo. Happy to answer anything about
> the methodology — including where it's weakest (README has a
> Limitations section that tries to get ahead of that).

---

## Lobsters

Same title and first-comment text as Hacker News — the plan notes `vet`
launched successfully here, so the audience already knows the category.

---

## r/programming

**Title:** same as Hacker News.
**Body:** same text as the HN first comment, posted as the submission body
up front (Reddit expects context in the post itself, not a follow-up
comment).

---

## X thread

1/ I analyzed 174 real install scripts — the kind you `curl | bash` for
rustup, docker, homebrew, ollama, k3s, tailscale.

55.7% download and execute a binary with **zero checksum or signature
verification.**

2/ `vet` proved people wanted this (1,000+ stars). Then it went quiet for
11 months.

So I picked it back up: bashlens — AST-based (tree-sitter-bash),
corpus-validated against 174 real installers.

3/ It never says a script is "safe." Static analysis can't prove that. It
reports what it observes and says plainly what it can't resolve.

[terminal screenshot/GIF of `bashlens compare bun deno uv` here — **not
generated yet**, see Open items below]

4/ Corpus + methodology + code, all public: github.com/rudranpatra/bashlens

---

## LinkedIn

> Most of us have piped a `curl | bash` install command into our shell
> without reading it first. I got curious how common that risk actually
> is, so I analyzed 174 real-world install scripts from tools people use
> every day (rustup, docker, homebrew, ollama, k3s, tailscale, and 168
> others).
>
> 55.7% download and execute a binary with no checksum or signature
> verification anywhere in the script.
>
> I built `bashlens` — an open-source, AST-based static analyzer
> (tree-sitter-bash) — to surface this automatically: network calls,
> privilege escalation, persistence, obfuscated execution, all reported
> explicitly, with no "safe/unsafe" verdict, because static analysis can't
> honestly make that promise.
>
> Corpus, methodology, and code: github.com/rudranpatra/bashlens

---

## Open items before posting

- [ ] `vet-run/vet` maintainer check (10 min, see top of this file)
- [x] Real terminal screenshots — generated with `termtosvg` from the actual
      `v0.1.0` release binary (not mocked): `docs/img/rustup.svg` (single
      scan) and `docs/img/compare.svg` (`bashlens compare` against bun/deno/uv
      live over the network). Both embedded in the README. For a raster PNG
      (X/LinkedIn prefer PNG over SVG in some clients), convert locally,
      e.g. `rsvg-convert docs/img/compare.svg -o compare.png` — not done
      here since no SVG-to-PNG renderer is available in this environment.
- [ ] Private review — explicitly skipped by user decision, not done
      (tracked in `STATUS.md`, not re-litigated here)
