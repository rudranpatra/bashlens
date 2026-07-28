#!/usr/bin/env python3
"""Crude Day 0 analysis over the collected installer corpus.

No parser. No Rust. Just counts and percentages so we can answer:
could we publish these findings tomorrow as "We analysed N install scripts."?
"""

import csv
import json
import re
from collections import Counter, defaultdict
from pathlib import Path

SCRIPTS_DIR = Path("corpus/scripts")
METADATA_PATH = Path("corpus/metadata.csv")
STATS_DIR = Path("corpus/stats")

PATTERNS = {
    "sudo": re.compile(r"\bsudo\b", re.IGNORECASE),
    "network": re.compile(r"\b(curl|wget)\b", re.IGNORECASE),
    "checksum": re.compile(
        r"\b(sha256sum|shasum|sha512sum|sha1sum|md5sum|cksum)\b", re.IGNORECASE
    ),
    "signature": re.compile(r"\b(gpg|gpgv|gpg2|gpgv2|cosign|minisign)\b", re.IGNORECASE),
    "piped_to_shell": re.compile(r"\|\s*(sh|bash)\b", re.IGNORECASE),
    "profile": re.compile(
        r"(\.bashrc|\.bash_profile|\.bash_login|\.zshrc|\.zprofile|\.profile|/profile\.d|\.config/fish)",
        re.IGNORECASE,
    ),
    "systemd": re.compile(r"\bsystemctl\b|/etc/systemd|/usr/lib/systemd", re.IGNORECASE),
    "eval": re.compile(r"\beval\b", re.IGNORECASE),
    "base64_decode": re.compile(r"base64\s+(?:-d|--decode)", re.IGNORECASE),
    "xxd": re.compile(r"\bxxd\b", re.IGNORECASE),
    "python_exec": re.compile(r"\b(python3?|python)\b.*\|\s*(sh|bash)", re.IGNORECASE),
}

URL_RE = re.compile(r"https?://([^/\s\"'<>]+)", re.IGNORECASE)


def read_script(path: Path) -> str:
    try:
        return path.read_text(errors="replace")
    except Exception:
        return ""


def analyze():
    STATS_DIR.mkdir(parents=True, exist_ok=True)

    rows = []
    if METADATA_PATH.exists():
        with METADATA_PATH.open(newline="") as f:
            rows = [r for r in csv.DictReader(f) if r.get("status") == "ok"]

    total = len(rows)
    if total == 0:
        print("No collected scripts to analyze. Run day0/collect.py first.")
        return

    pattern_counts = Counter()
    scripts_with = defaultdict(list)
    domain_counter = Counter()
    verification_any = []
    network_any = []

    for row in rows:
        name = row["name"]
        text = read_script(SCRIPTS_DIR / name)
        has_network = bool(PATTERNS["network"].search(text))
        has_checksum = bool(PATTERNS["checksum"].search(text))
        has_sig = bool(PATTERNS["signature"].search(text))
        has_verification = has_checksum or has_sig

        if has_network:
            network_any.append(name)
        if has_verification:
            verification_any.append(name)

        for label, pat in PATTERNS.items():
            if pat.search(text):
                pattern_counts[label] += 1
                scripts_with[label].append(name)

        for m in URL_RE.finditer(text):
            domain_counter[m.group(1).lower()] += 1

    # Network + no explicit verification is the strongest headline candidate.
    network_without_verification = [
        name for name in network_any if name not in verification_any
    ]

    # Top domains (sanity check for unknown/suspicious hosts)
    top_domains = domain_counter.most_common(20)

    stats = {
        "total_scripts": total,
        "patterns": {k: pattern_counts[k] for k in PATTERNS},
        "percentages": {
            k: round(100 * pattern_counts[k] / total, 1) for k in PATTERNS
        },
        "network_without_verification": {
            "count": len(network_without_verification),
            "percent": round(100 * len(network_without_verification) / total, 1),
            "examples": network_without_verification[:10],
        },
        "top_domains": top_domains,
    }

    stats_path = STATS_DIR / "day0_stats.json"
    with open(stats_path, "w") as f:
        json.dump(stats, f, indent=2, sort_keys=True)

    report_path = STATS_DIR / "day0_report.md"
    with open(report_path, "w") as f:
        f.write("# Day 0 — Install Script Corpus Findings\n\n")
        f.write(f"**Scripts collected:** {total}\n\n")
        f.write("## Behaviour prevalence\n\n")
        f.write("| Behaviour | Count | % |\n|---|---:|---:|\n")
        for label in PATTERNS:
            count = pattern_counts[label]
            pct = stats["percentages"][label]
            f.write(f"| {label} | {count} | {pct}% |\n")

        f.write("\n## Headline signal\n\n")
        f.write(
            f"- **{stats['network_without_verification']['percent']}%** "
            f"({len(network_without_verification)}/{total}) of scripts "
            f"download over the network without an explicit checksum or signature check.\n"
        )
        if network_without_verification:
            f.write("\nExamples (network, no verification):\n\n")
            for n in network_without_verification[:10]:
                f.write(f"- `{n}`\n")

        f.write("\n## Gate decision\n\n")
        strong = stats["network_without_verification"]["percent"] >= 30.0
        obfuscation_pct = stats["percentages"].get("eval", 0) + stats["percentages"].get(
            "base64_decode", 0
        )
        moderate = obfuscation_pct >= 5.0 or stats["network_without_verification"]["percent"] >= 10.0

        if strong:
            f.write(
                "**GO.** ≥30% download without verification — a strong, publishable finding.\n"
            )
        elif moderate:
            f.write(
                "**MAYBE.** The signal is moderate; expand the corpus or dig for a specific surprising outlier.\n"
            )
        else:
            f.write(
                "**NO-GO.** The corpus is not yet producing a headline. Stop or pivot.\n"
            )

        f.write("\n## Top domains referenced\n\n")
        for domain, count in top_domains:
            f.write(f"- {domain}: {count}\n")

    print(f"Wrote {stats_path}")
    print(f"Wrote {report_path}")
    print(f"\nCollected {total} scripts")
    print("Key percentages:")
    for label in PATTERNS:
        print(f"  {label:20s}: {stats['percentages'][label]:5.1f}%")
    print(f"\nNetwork w/o verification: {stats['network_without_verification']['percent']:.1f}%")
    print("\nGate:", "GO" if strong else ("MAYBE" if moderate else "NO-GO"))


if __name__ == "__main__":
    analyze()
