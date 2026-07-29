# Security Policy

## Reporting a vulnerability in bashlens itself

If you find a security issue in the `bashlens` binary or its detection
engine (e.g. a crash on adversarial input, a way to make it report false
information, or a supply-chain issue in a dependency), please report it
privately rather than opening a public issue:

- Preferred: GitHub's private vulnerability reporting, if enabled on this
  repo (Security tab → "Report a vulnerability").
- Otherwise: open an issue asking for a private contact method, without
  including exploit details in the issue itself.

Please allow a reasonable window to fix the issue before public disclosure.

## What this is *not* for

Findings `bashlens` surfaces *about* a third-party install script (e.g. "X's
installer downloads a binary with no checksum") are the intended, public
output of this tool - discuss those in the open, not privately. If you
believe a specific installer's behavior is malicious rather than merely
undocumented, disclose that to the installer's own maintainer first (see
the README's methodology notes on responsible disclosure), not here.

## Supported versions

Pre-1.0: only the latest tagged release is supported. There is no LTS
branch yet.
