# Day 0 — Install Script Corpus Findings

**Scripts collected:** 174

## Behaviour prevalence

| Behaviour | Count | % |
|---|---:|---:|
| sudo | 66 | 37.9% |
| network | 164 | 94.3% |
| checksum | 45 | 25.9% |
| signature | 32 | 18.4% |
| piped_to_shell | 43 | 24.7% |
| profile | 46 | 26.4% |
| systemd | 17 | 9.8% |
| eval | 30 | 17.2% |
| base64_decode | 4 | 2.3% |
| xxd | 0 | 0.0% |
| python_exec | 0 | 0.0% |

## Headline signal

- **55.7%** (97/174) of scripts download over the network without an explicit checksum or signature check.

Examples (network, no verification):

- `rustup`
- `bun`
- `deno`
- `ollama`
- `homebrew`
- `starship`
- `nvm`
- `volta`
- `flyctl`
- `pnpm`

## Gate decision

**GO.** ≥30% download without verification — a strong, publishable finding.

## Top domains referenced

- github.com: 457
- raw.githubusercontent.com: 39
- www.apache.org: 36
- api.github.com: 34
- packagecloud.io: 17
- stackoverflow.com: 14
- pkgs.tailscale.com: 13
- docs.docker.com: 12
- getmic.ro: 10
- unlicense.org: 10
- gist.github.com: 9
- download.docker.com: 7
- docs.nvidia.com: 7
- docs.brew.sh: 7
- pkgs.netbird.io: 7
- git.io: 7
- support.apple.com: 6
- discord.gg: 6
- releases.astral.sh: 6
- get.docker.com: 6
