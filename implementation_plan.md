# `bashlens` + Install-Script Corpus — Project Brief

**Status:** v1.0 — frozen. Next artifact is the Day 0 corpus, not another revision of this document.
**Scope:** one person, one weekend to v0.1
**Date:** July 2026

> **Internal planning note only — never say this publicly.** No one can predict virality, and a stated number is either a target you visibly miss or a brag that reads badly either way. For internal calibration purposes only, comparable solo security tools in this niche land 2,500–5,000 stars over 3–6 months (see Section 3 for the reference repos). This number appears nowhere in the README, launch post, or any public copy.

> **Internal planning note only — ecosystem context.** `bashlens` may sit within a broader body of work on trust in automated execution (other projects, other questions — not named here since this is a private document, not a public one). If that's the intent, one quiet line belongs in the README's "about" section — something like *"part of an ongoing line of work on trust in automated execution"* — enough for anyone who later connects the dots, without disclosing a roadmap. A public diagram naming multiple unreleased projects and ending in a governance/compliance product is a different document for a different audience (investors, not developers deciding whether to star a tool), and it tells competitors what's coming before any of it exists. Keep that framing here, not in anything `bashlens` ships. This also sits in tension with the original brief's "no branding overlap" requirement — worth resolving on purpose if the plan has changed, not by default.

---

## 1. The two ideas, and why they are one project

| # | Idea | Defender status | Launch spike | Sustained path | 6-month estimate | Call |
|:--|---|---|---:|---|---:|---|
| 1 | `bashlens` + install-script corpus | `vet` dead 11 months, 1,090 proven pull, no successor | 800–1,500 | Repeat use per script, CI action, quarterly corpus re-run = 3 launch moments | **2,500–5,000** | **BUILD** |
| 2 | Corpus-only play | none | 1,000–3,000 | Citations, but no tool means no repeat use | 3,000–15,000 | **Merge into #1** |

**These are not alternatives. They are two halves of one launch.**

- The **corpus** is the launch asset. A research finding gets to the front page; a tool announcement usually does not.
- The **tool** is the retention asset. A dataset is read once and cited; a CLI is run every time someone installs something.

Separately each one misses the target. The corpus has no repeat usage, so it spikes and flatlines. The tool has no headline, so it never spikes at all. Combined, the corpus supplies the launch and the tool supplies the curve.

**The product is three layers, not one binary.** Corpus is what gets you found. The CLI is what gets you re-run. The Transparency Index (Section 8) is what gets you cited months later, by people who never touch the tool at all. Build order should follow that: corpus first, gated on Day 0; CLI second, as the thing that actually produces personal, screenshottable results (`npx bashlens compare bun deno uv` doesn't exist without it); Index third, as a presentation layer over data both the other two already produce. None of the three is a distant afterthought — a corpus with no CLI is a blog post nobody can independently verify, and an Index with no CLI has nothing feeding it.

---

## Why now

A one-page, visual case for urgency rather than pure utility — this belongs near the top of the README, not buried:

| Year | State |
|---|---|
| 2023 | Developers trusted install scripts by default |
| 2024 | Supply-chain attacks on the npm/PyPI ecosystem increased sharply |
| 2025 | Developer tooling — and the number of install scripts in circulation — exploded |
| 2026 | AI-generated install instructions became common, and coding agents now run `curl \| bash` on a developer's behalf, unsupervised |
| **Now** | We still execute `curl \| bash` blindly |

That last row is the point. Every year in the table made the practice riskier; the practice itself hasn't changed at all.

**The trust boundary has shifted, and it's worth one paragraph, not a section:**

> Twenty years ago, `curl | bash` was a decision a developer consciously made. Today, coding agents routinely propose installation commands as part of larger workflows — and the data already cited in this document (Section 3) shows developers approve roughly 93% of what an agent proposes, reading less carefully with each approval. The question has quietly changed from "do I trust this script" to "do I trust every script my AI decided was fine to run." `bashlens` exists because humans are no longer the only ones deciding to execute install scripts — they're increasingly the ones rubber-stamping the decision after the fact.

Grounded in the approval-fatigue figure already in the doc rather than an unqualified claim that agents execute scripts "automatically" — most harnesses still gate execution behind approval by default, and a security-literate reader will catch an overclaim in the opening paragraph faster than anywhere else in the document.

---

## 2. What it does

### `bashlens` — the tool

A single static binary that reads a shell install script and reports what it will do to your machine, before you pipe it into your shell.

```
npx bashlens https://get.example.dev/install.sh
```

No install. No config. No API key. No account. One network request — fetching the URL you supplied.

It reports six classes of behaviour:

| Class | What it surfaces |
|---|---|
| **Privilege** | Every `sudo` invocation and the exact path it targets |
| **Persistence** | Writes to shell profiles, `profile.d`, systemd user units, cron, launchd |
| **Network** | Every URL, its role (download / pipe-to-shell / POST), and what is sent outbound |
| **Verification** | Whether a downloaded binary is checked against a hash or signature before execution |
| **Obfuscation** | `eval`, `base64 -d`, `xxd -r`, rot13, second-stage fetches, `$(...)` fed from network responses |
| **Unresolvable** | Constructs that static analysis genuinely cannot resolve, reported explicitly rather than silently skipped |

**It never says a script is safe.** Bash is not statically analysable in the general case. The tool describes what it observes and flags what it cannot. A verdict would be dishonest, and — for a founder with a security reputation to protect — a liability the first time it is bypassed.

### The corpus — the research

A regression suite of roughly 200 real-world installers from tools developers actually use, stored with hashes, and the aggregate statistics derived from them.

Two functions:
1. **Engineering.** Every detection rule is validated against 200 real scripts instead of hand-written fixtures. This is what makes the tool correct.
2. **Distribution.** The aggregate statistics are the launch. *"I analysed the install scripts of 200 tools you have piped into bash. N% verify nothing."*

---

## 3. Why this slot is open

Verified against the GitHub API, July 2026.

| Repo | Stars | Last push | Reading |
|---|---:|---|---|
| `vet-run/vet` | **1,090** | **2025-08-20** | Proven demand. Abandoned 11 months. Written in Shell |
| `chriswessels/bashtion` | **2** | 2025-11-21 | Pushed once, on the day it was created |
| `jrcribb/vet` (only fork past abandonment) | **0** | 2026-03-26 | No successor emerged |
| New 2026 entrants to the niche | **0** | — | One repo, zero stars |

**What this configuration means:**

- **Demand is proven, not hypothetical.** `vet` reached 1,090 stars in roughly two months. The pitch converts. This is the single most valuable fact in the brief — most new tools are gambling on whether anyone cares, and this one is not.
- **There is no active defender.** No commits in 11 months. Seven open issues, most with zero maintainer replies. No fork has taken over.
- **A prior attempt failed on distribution, not architecture.** `bashtion` chose a sound approach (Rust, tree-sitter) and got two stars because it made no distribution effort. The lesson is about launch, not stack.
- **Two features are pre-validated and unbuilt.** `vet`'s abandoned issue queue contains *"Introduce Installer Manifests for Download Verification"* and *"Review new scripts by default"* — upvoted, unanswered, and directly buildable.

### Calibration

The reference points that set realistic expectations:

| Repo | Stars | Age | Rate |
|---|---:|---:|---:|
| `mukul975/Anthropic-Cybersecurity-Skills` (a curated collection) | 26,755 | 153d | 175/day |
| `NVIDIA/SkillSpector` | 13,859 | 129d | 107/day |
| `nolabs-ai/nono` | 3,182 | 178d | 18/day |
| `sheeki03/tirith` | 2,626 | 176d | 15/day |
| ripgrep (lifetime) | 66,599 | 10y | 17.6/day |
| Ollama (lifetime) | 177,061 | 3y | 157/day |

Two conclusions. First, 2,500–5,000 in six months is 15–55 stars/day, which matches what solo developers in this space actually achieve. Second, the best-performing repo in the entire 2026 AI-security cohort is a **curated collection**, not a tool — which is the empirical basis for leading with the corpus.

---

## 4. Advantages

### Against `vet` (the abandoned incumbent)

**The pitch is the corpus, not the parser.** Anyone can clone a tree-sitter integration in a weekend. Nobody can clone 200-plus curated, hashed, re-run installers overnight. That is the actual moat, and it is what should be foregrounded everywhere — README hook, launch post, social copy. "Largest corpus of install scripts ever analysed" beats "written in Rust" as a reason to care, every time.

Engineering choices still matter for correctness and trust, they just don't belong in the pitch:

| | `vet` | `bashlens` |
|---|---|---|
| Distribution | Shell script | Single static binary + `npx` + Homebrew |
| Analysis | shellcheck + diff-since-last-run | AST-based, six behaviour classes |
| Validation | hand-written cases | corpus-validated |
| Verification detection | requested, never built | shipped in v0.1 |
| Comparative context | none | percentile against the corpus |
| Maintenance | none since Aug 2025 | active |

Keep the technical detail — tree-sitter, musl, single-binary distribution — in a "how this works" section for contributors and the technically curious. It's a legitimate answer to "why should I trust a security tool," but it is not the hook.

**The README should open with a question, not a description.** Not *"analyse install scripts before running them"* — that's a feature list. Open with:

> *"You wouldn't run a random binary. So why do we run random install scripts?"*

It's a fair rhetorical hook, not a strictly defensible claim — plenty of people do run unverified binaries too — but that's what makes it work as an opener: it names an inconsistency the reader recognizes in themselves before the tool has done anything. Everything downstream — the corpus stat, the percentile bars, the compare command — earns its place by cashing out that one sentence in ten seconds.

### Structural advantages

- **Near-zero trial barrier.** `npx bashlens <url>` requires no install, no signup, no key. Nothing between a curious reader and the payoff.
- **Repeat usage is inherent.** Unlike a one-shot audit tool, there is a fresh reason to run it every time you install something. This is what produces a curve instead of a spike.
- **Describes rather than judges.** No safe/unsafe verdict means no headline the first time someone bypasses it. For a founder whose credibility is in security, this is the difference between an asset and a liability.
- **The corpus is not clonable in a weekend.** Anyone can copy the parser. Reproducing 200 curated installers plus the statistics is a week of work, which is your moat.
- **Pre-empts its own strongest criticism.** The `UNRESOLVABLE` output section concedes the "static bash analysis is defeatable" objection louder than any critic can raise it. Conceded limitations do not become top comments.
- **The share trigger writes itself.** Users will run it against tools they already use and post the gap between documented and actual behaviour. That is a content engine you do not have to operate.

---

## 5. How it helps — for whom

**For developers.** Compresses fifteen minutes of careful reading into five seconds. Nobody reads a 400-line installer with three levels of function indirection; everybody feels vaguely bad about that. This closes the gap between what people know they should do and what they will actually do.

**For CI and teams.** `--quiet --fail-on undocumented-network,obfuscated-exec` fails a build when a vendored installer starts phoning home. Turns a one-time check into continuous supply-chain monitoring.

**For maintainers.** `--markdown` produces output designed to be pasted into a polite issue. Most undocumented telemetry is an oversight, not an attack, and the tool should make the good-faith path the easy one.

**For the founder — the actual objective.**
- Establishes credibility in supply-chain security through published research rather than assertion.
- Produces a citable dataset — a durable asset that survives version churn, unlike any parser.
- Creates recurring inbound: every quarterly corpus re-run is a fresh post with fresh numbers and no new engineering.
- Carries no branding overlap with existing commercial products (AI runtime governance, eBPF, threat recon). Adjacent enough to be credible, separate enough to be a clean reputation play.

---

## 6. Build plan — weekend v0.1

### Stack decisions, and the reasoning

- **Rust, single static binary (musl).** The differentiator against `vet`.
- **`tree-sitter-bash` for parsing. Do not hand-roll.** Writing a shell parser is a multi-week sinkhole disguised as a two-day task, and it is the single most likely way this weekend dies. tree-sitter gives a real AST in an afternoon.
- **Rules as data, not code.** Each detection is a tree-sitter query (`.scm`) plus metadata plus two fixtures (one passing, one failing). Makes the contribution story real; allows new detections without touching core logic.
- **Distribution from day one.** `cargo install`, Homebrew tap, prebuilt binaries via GitHub Actions release, thin npm wrapper for `npx`. `bashtion` failed here — do not repeat it.

### Day 0 (half day) — the go/no-go gate

**Build the corpus before writing any tool code.**

1. Scrape GitHub code search for READMEs containing `curl … | sh` / `| bash` patterns; dedupe URLs.
2. Fetch ~200 scripts. Store with SHA-256 and source attribution.
3. Add high-profile installers by hand: rustup, nvm, bun, deno, uv, ollama, tailscale, docker, k3s, nix, starship, homebrew.
4. Run crude grep-level statistics — no parser needed yet.

**The real test: would this be worth posting even if the tool did not exist.** Not "interesting percentages" and not "one gotcha on a named popular project" — the latter turns a research post into a public callout of a specific maintainer, which creates a responsible-disclosure obligation before you can publish anything at all, and it overfits the whole gate to a single lucky outlier. The bar is a finding that stands on its own as a post, with the CLI as the methods section underneath it:

| Threshold | Signal |
|---|---|
| ≥30% download and execute a binary with no checksum or signature verification | Strong |
| ≥10% contact a domain not mentioned in their documentation | Strong |
| ≥5% contain obfuscated execution | Moderate |
| Any single well-known tool doing something genuinely surprising | Sufficient alone, but disclose to the maintainer before publishing — see FAQ commitment in Section 2 |

**Ship only if at least one holds and you can say, honestly, that you'd share that specific number even with no code attached to it.** Miss it → stop. Pivot to the approval-fatigue meter (zero competitors, still unclaimed). Half a day spent instead of a weekend.

This gate exists because the alternative is discovering on day three that there is no headline, with the sunk cost already paid and every incentive to talk yourself into numbers that are not interesting.

### Day 1 — pipeline

- tree-sitter-bash integration, AST walk, rule-dispatch scaffold.
- Three highest-signal rules end-to-end: **verification presence first** (most shareable), then sudo targets, then persistence writes.
- Ugly output is fine. Working pipeline is the deliverable.

### Day 2 — validate against reality

- Run all 200 corpus scripts. This will expose rule bugs; that is the point of ordering it this way.
- Fix. Add obfuscated-exec and unresolvable-construct detection.
- Generate the aggregate statistics CSV. **These numbers are the launch post.**

### Day 3 — ship

- **Report design. Spend disproportionate time here — the terminal output is the product.** Concretely, the output should look like this, not a wall of text:

```
$ npx bashlens https://get.example.dev/install.sh

  Network       ███████░░░  high
  Privilege     ██░░░░░░░░  low
  Persistence   █████░░░░░  medium
  Verification  ❌ none

  Risk percentile: 94th
  Compared against 418 installers in the corpus
```

A bar chart and a percentile number is a screenshot. A paragraph of findings is not. This is the single highest-leverage design decision in the whole build — it is the difference between people reading the tool and people posting the tool.
- `--json`, `--markdown`, `--quiet` / `--fail-on`.
- README: hook line above the fold, example output before install instructions, no safe verdict anywhere.
- Release automation, npx wrapper, Homebrew formula.

### v0.1 detection set

1. **Verification presence** — is a `sha256sum` / `shasum` / `gpg` / `cosign` check present downstream of a download? *Build first.*
2. **Privilege** — `sudo` invocations and target paths.
3. **Persistence** — redirect/append nodes hitting shell profiles, `profile.d`, systemd, cron, launchd.
4. **Network** — every URL, role-classified, with outbound payload contents.
5. **Obfuscated exec** — `eval`, `base64 -d`, `xxd -r`, rot13, network-fed `$(...)`.
6. **Unresolvable constructs** — variable-derived paths and URLs, reported explicitly.

### Explicitly cut from v0.1

- `--sandbox` (container execution + syscall trace + filesystem diff) — **branch it with a public merge date. This is launch moment #3.**
- Windows / PowerShell — not planned at all.
- zsh / fish beyond best-effort, clearly labelled.
- Remediation advice.
- **Any safe/unsafe verdict. Permanently.**

### Two things nobody else has

- **The two abandoned `vet` issues:** installer manifests / download verification, and review-by-default. Pre-validated demand, zero competition to satisfy it.
- **`--compare`:** *"This installer sits in the 88th percentile for risky behaviours across 200 corpus installers."* Requires the research, so it cannot be cloned in a weekend, and it gives the tool a reason to be re-run.
- **`bashlens compare <tool> <tool> <tool>`:** runs the same analysis across several installers side by side —

  ```
  $ npx bashlens compare bun deno uv

              Verification   Network   Persistence
  bun         98th %ile      34th      12th
  deno        91st %ile      41st      8th
  uv          76th %ile      22nd      15th
  ```

  This is the feature that turns a personal lookup into a discussion. A single-URL scan tells you about one tool; a comparison invites an argument about which of three popular tools you already use is the risky one. Cheap to build — it's the existing single-URL analysis run N times — and it's the most repeatable share trigger in the whole project, because everyone has an opinion about their package manager.

---

## 7. Launch

**Lead with the finding, not the tool.** *"I analysed the install scripts of 200 tools you have piped into bash. N% verify nothing."* The tool is the methods section.

| Channel | Notes |
|---|---|
| **Lobsters** | `vet` launched successfully here. Proven audience for exactly this |
| **Hacker News** | Findings post, not Show HN. Tue–Thu, ~8am ET |
| **r/programming** | Same framing |
| **X** | Thread with terminal recording |

**Pre-launch, ten minutes:** open an issue on `vet-run/vet` asking whether the maintainer intends to return.
- If they are done → you have clearance to tell the story below in full.
- If they are returning → credit them and soften the framing; do not declare victory over an active project.

**Name `vet` in the README — do not hide it.** Hiding well-known prior art is worse than any risk of naming it: someone will surface it in the comments regardless, and being seen to have hoped nobody would notice costs more credibility than an honest account of the history. The story is the asset:

> *"`vet` proved people wanted this. Then nobody built it for eleven months. So we did."*

That is a better opening line than anything invented from scratch, and it pre-empts the "why didn't you just use vet" comment by answering it before anyone asks.

---

## 8. Roadmap to 5,000

Three launch moments, not one. This is the step most solo founders skip, and it is why good tools plateau at 800 stars.

| Month | Moment | Work required | Expected |
|:--:|---|---|---:|
| **0** | Corpus findings + tool launch | the weekend | 800–1,500 |
| **1–2** | GitHub Action for CI; external rule contributions | small | +400–800 |
| **2–3** | `--sandbox` merges; second post comparing static analysis vs. observed behaviour | 1 week | +600–1,200 |
| **4–6** | Corpus re-run: *"Installer security, six months later"* | 1 day | +500–1,500 |
| | | **Cumulative** | *(internal calibration only — see header note)* |

### The distribution flywheel

Do not run a full research post weekly — installer behaviour doesn't change week to week, and a repeated "still 41%" post reads as content-farming, which costs the exact credibility this project depends on. Instead, separate the mechanical from the editorial:

- **Weekly, automated:** re-run the corpus scan as a scheduled job, update a living scorecard — see the Installer Transparency Index below. Near-zero cost once built.
- **Opportunistic, editorial:** a full blog/HN post only when the index produces something genuinely worth saying — a well-known tool fixing or breaking its installer, a milestone corpus size, a notable outlier.

### The Installer Transparency Index — not a leaderboard, a scorecard

A ranked "worst installers" list is read once and forgotten. Build it instead as a per-tool scorecard people can cite and point to, on the model of SSL Labs' grades or the OpenSSF Scorecard — both of which converted into standing reference points that projects actively work to improve, not one-off shaming lists:

| Tool | Verification | Network | Persistence | Privilege | Trend | Last changed |
|---|---|---|---|---|---|---|
| example-cli | ✅ signed | 1 domain | none | none | ↑ improved | 2026-06-02 |

Same underlying data as a leaderboard, different incentive structure: a maintainer with a bad score has something concrete to fix and a public before/after to point at once they do — "we went from a C to an A" is its own press cycle, and it's one you don't have to generate yourself.

This gets the "always current" feel of a weekly cadence without manufacturing eleven consecutive weeks of nothing.

### Beyond bash — roadmap, not v0.1

The stronger long-term frame is "install behaviours," not "bash specifically" — `curl | python`, `curl | zsh`, `curl | fish`, and PowerShell's `iex(...)` are the same pattern in different languages. This belongs in the README as a stated direction ("starting with the most common vector; more shells on the roadmap"), not in the weekend build. Adding PowerShell in particular is a different parser, a different audience, and close to a different product — it directly contradicts the "don't over-invest in parsing, the corpus is the moat" principle above if pulled into v0.1. Ship bash/sh first; expand once the corpus and the core loop are proven.

The final row is the real asset. **A corpus you can re-measure is a repeatable content engine** — every quarter it produces a fresh post with fresh numbers and effectively no new engineering. That is the mechanism that converts a launch spike into a six-month curve, and it is what delivers the target.

---

## 9. Risks, stated honestly

| Risk | Severity | Mitigation |
|---|---|---|
| **Corpus turns out boring** | High probability, low cost | Day 0 gate. Half a day lost, pivot to approval-fatigue meter |
| Static analysis defeatable by a motivated attacker | Certain | True and unfixable. Stated in the README threat model. Never claim protection, only visibility |
| `vet` maintainer returns | Low | Ten-minute pre-launch check |
| "Just read the script" as top comment | Likely | Answer it in the README FAQ before it is asked |
| tree-sitter edge cases on exotic scripts | Medium | Corpus catches them; unresolvable constructs reported rather than guessed |
| Someone clones the parser | Medium | The corpus is the moat, not the parser |
| Maintenance burden of rule requests | Medium | Rules-as-data plus fixture requirement makes contribution the default path |

**The most likely failure is Day 0.** Design for it and it costs half a day. Skip the gate and it costs the weekend plus a launch built on numbers you had to squint at.

---

## 10. Success / failure — decided in advance, anchored by when it's actually checkable

The point of this section is to stop the project from being judged on feelings after the fact. A flat list invites exactly that: you hit the fast criteria, feel good, and never circle back on the slow ones, or you panic in week one over a criterion that was never supposed to resolve that quickly. So each item is tied to the moment it can honestly be evaluated.

**At Day 0 (hours, not days) — go/no-go, per Section 6's gate:**
- Success: the corpus produces at least one finding you would honestly post even if the tool didn't exist.
- Failure: nothing in the data would earn a post on its own. **Kill here.** Do not spend the weekend trying to rescue a weak thesis with better engineering.

**At launch (week 1):**
- Success: a developer can go from `npx bashlens <url>` to understanding the installer in under 10 seconds, with zero explanation needed first.
- Failure: the tool requires you to explain why it's useful before someone gets it, or the output isn't meaningfully faster than just reading the script.

**First month:**
- Success: at least one maintainer, contacted through the disclosure process in Section 2's FAQ, uses a finding to change their installer.
- Failure: the project produced launch-day curiosity and nothing that brings anyone back a second time.

**Ongoing, operational (not a market signal, a build-completeness check):**
- Success: the Transparency Index regenerates automatically from the corpus with no manual step.
- Failure: keeping it current requires you personally, every time — which quietly turns Section 8's flywheel back into manual labor.

---

## Before you stop planning — a checklist, not a delay

Two items from earlier rounds don't go away just because the document is frozen. Handle them in parallel with Day 0, not after:

- [ ] **Ask the `vet` maintainer** whether they intend to return (Section 7). Ten minutes. Blocks how the launch story gets told, not the build itself.
- [ ] **Decide the brand-portfolio question** (Phantom / Crucible / Flight, if that's still live) before README/launch copy is final. Doesn't block Day 0 or the CLI build — only blocks the words in the final pitch.

Neither blocks starting the corpus scrape tomorrow.

---

## Freeze

**This document is v1.0.** Recommendation, and I agree with it: don't touch it again until the Day 0 gate produces an actual result — not on a calendar timer, since the gate itself is designed to resolve in hours, and there's no reason to sit on a clear go/no-go for a week just because a week sounds disciplined. The next artifact is one of:

1. The Day 0 corpus and its results.
2. The first terminal output.
3. The README, with the hook and a real example.

Further planning past this point has diminishing returns. The document is sufficient to execute against.

---

## 11. One-line summary

Resurrect a proven 1,000-star category whose defender walked away 11 months ago — but launch it as research rather than as a tool, because in this niche a curated dataset outperformed NVIDIA by a factor of two.