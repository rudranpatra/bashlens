#!/usr/bin/env python3
"""Fetch the Day 0 install-script corpus.

Sources (corpus/sources.csv) are tried first; then GitHub code search is used
to backfill up to a few hundred install.sh files.  Everything is stored as
shell scripts in corpus/scripts/ with a metadata.csv sidecar.
"""

import csv
import hashlib
import json
import os
import time
import shlex
import subprocess
import urllib.parse
from pathlib import Path

SCRIPTS_DIR = Path("corpus/scripts")
STATS_DIR = Path("corpus/stats")
METADATA_PATH = Path("corpus/metadata.csv")
SOURCES_PATH = Path("corpus/sources.csv")

# Be polite to the hosts we are scraping.
USER_AGENT = "bashlens-day0/0.1 (+research corpus; no abuse)"
REQUEST_TIMEOUT = 30
SLEEP = 0.25

# Some "install scripts" (Miniconda/Mambaforge-style makeself installers) are a
# small shell header followed by a multi-hundred-MB binary payload appended to
# the same file. Only the header is ever analyzed as shell, so anything past
# this cap is truncated rather than stored - it's regex noise, and storing full
# binary payloads in a "script corpus" would balloon repo size for zero signal.
MAX_SCRIPT_BYTES = 262_144


def mkdirs():
    SCRIPTS_DIR.mkdir(parents=True, exist_ok=True)
    STATS_DIR.mkdir(parents=True, exist_ok=True)
    METADATA_PATH.parent.mkdir(parents=True, exist_ok=True)


class NotAScriptError(Exception):
    pass


def looks_like_html(data: bytes) -> bool:
    head = data[:1024].lstrip().lower()
    return head.startswith(b"<!doctype") or head.startswith(b"<html") or (
        b"<head" in head[:200] and b"<body" not in head[:200] and head.startswith(b"<")
    )


def fetch(url: str) -> bytes:
    cmd = [
        "curl",
        "-fsSL",
        "--max-time",
        str(REQUEST_TIMEOUT),
        "-A",
        USER_AGENT,
        "--",
        url,
    ]
    result = subprocess.run(cmd, capture_output=True, check=True)
    data = result.stdout
    if looks_like_html(data):
        raise NotAScriptError(f"response looks like an HTML page, not a shell script ({len(data)} bytes)")
    return data


def fetch_and_truncate(url: str) -> tuple[bytes, int]:
    """Returns (possibly-truncated data, original byte length)."""
    data = fetch(url)
    original_len = len(data)
    if original_len > MAX_SCRIPT_BYTES:
        data = data[:MAX_SCRIPT_BYTES]
    return data, original_len


def read_csv_sources():
    rows = []
    if not SOURCES_PATH.exists():
        return rows
    with SOURCES_PATH.open(newline="") as f:
        reader = csv.DictReader(f)
        for row in reader:
            name = row.get("name", "").strip()
            url = row.get("url", "").strip()
            if name and url:
                rows.append((name, url))
    return rows


def github_search_install_scripts(per_page: int = 100, pages: int = 2):
    """Return (name, raw_url) tuples from GitHub code search."""
    results = []
    for page in range(1, pages + 1):
        query = "filename:install.sh language:Shell"
        encoded = urllib.parse.quote(query)
        api_url = (
            f"https://api.github.com/search/code?q={encoded}"
            f"&per_page={per_page}&page={page}"
        )
        cmd = [
            "curl",
            "-fsSL",
            "--max-time",
            str(REQUEST_TIMEOUT),
            "-A",
            USER_AGENT,
            "-H",
            "Accept: application/vnd.github+json",
            api_url,
        ]
        try:
            result = subprocess.run(cmd, capture_output=True, check=True)
            data = json.loads(result.stdout.decode("utf-8"))
        except subprocess.CalledProcessError as e:
            stderr = e.stderr.decode("utf-8", errors="replace")[:200]
            print(f"GitHub search page {page} failed: {stderr}")
            break
        except Exception as e:
            print(f"GitHub search page {page} failed: {e}")
            break

        items = data.get("items", [])
        if not items:
            break

        for item in items:
            repo = item.get("repository", {}).get("full_name", "")
            path = item.get("path", "")
            branch = item.get("repository", {}).get("default_branch", "master")
            if not repo or not path:
                continue
            raw_url = f"https://raw.githubusercontent.com/{repo}/{branch}/{path}"
            # Derive a unique filesystem name.
            base = Path(path).name
            slug = repo.replace("/", "_").replace("-", "_")
            name = f"{slug}_{base}"
            if not name.endswith(".sh"):
                name += ".sh"
            results.append((name, raw_url))
    return results


def collect():
    mkdirs()
    sources = read_csv_sources()
    print(f"Loaded {len(sources)} sources from {SOURCES_PATH}")

    # Backfill from GitHub; this may be rate-limited and fail gracefully.
    try:
        gh = github_search_install_scripts()
        print(f"GitHub search returned {len(gh)} candidates")
        sources.extend(gh)
    except Exception as e:
        print(f"GitHub search unavailable, continuing with CSV only: {e}")

    rows = []
    seen_names = set()
    for name, url in sources:
        # Deduplicate on name; keep the first occurrence.
        if name in seen_names:
            continue
        seen_names.add(name)

        path = SCRIPTS_DIR / name
        try:
            data, original_len = fetch_and_truncate(url)
            sha256 = hashlib.sha256(data).hexdigest()
            with open(path, "wb") as f:
                f.write(data)
            note = ""
            if original_len > len(data):
                note = f"truncated to {len(data)} bytes (original {original_len})"
            rows.append([name, url, "ok", len(data), sha256, note])
            suffix = f"  [{note}]" if note else ""
            print(f"OK  {name:40s} {len(data):>7d} bytes{suffix}")
        except Exception as e:
            rows.append([name, url, "error", 0, "", str(e)])
            print(f"ERR {name:40s} {e}")
            # A prior run may have written this file when the source last
            # succeeded. If it no longer succeeds, the stale copy must not
            # linger on disk - the CLI's corpus scan reads every file in this
            # directory regardless of what metadata.csv says, so a leftover
            # file for a now-failing source would silently pollute both the
            # published stats and the risk-percentile baseline.
            if path.exists():
                path.unlink()
        time.sleep(SLEEP)

    with open(METADATA_PATH, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["name", "url", "status", "size", "sha256", "error"])
        w.writerows(rows)

    ok = sum(1 for r in rows if r[2] == "ok")
    print(f"\nCollected {ok}/{len(rows)} scripts into {SCRIPTS_DIR}")
    print(f"Metadata written to {METADATA_PATH}")


if __name__ == "__main__":
    collect()
