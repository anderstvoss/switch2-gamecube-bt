# Security Policy

## Reporting Vulnerabilities

Please do not disclose suspected vulnerabilities in public issues or pull
requests.

Use GitHub private vulnerability reporting for
`anderstvoss/switch2-gamecube-bt` when available. If that is not available,
contact the maintainer directly and include enough detail to reproduce the
issue safely.

## Defensive Posture

This repository uses local pre-commit and pre-push checks, secret scanning with
gitleaks, custom blockers for risky files and local paths, Rust lint/test gates,
cargo-deny, cargo-audit, CodeQL, OpenSSF Scorecard, SBOM generation, Dependabot,
and branch-protection controls for the project repository.
