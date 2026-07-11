# switch2-gamecube-bt

Rust workspace for Switch 2 GameCube Bluetooth experiments.

The repository includes pinned local checks, committed git hook wrappers,
secret and artifact blockers, Rust lint/test gates, supply-chain checks,
CodeQL, OpenSSF Scorecard, SBOM generation, Dependabot, issue/PR templates, and
security reporting documentation.

## AI Usage Disclosure

This project was authored with extensive use of LLM/Coding Agents.

## Local Setup

Install the local tooling once per machine. Install `uv` using the official
installation method for your platform, then use it to install `pre-commit`:

Install gitleaks separately and make it available on `PATH`.

```bash
uv tool install pre-commit
cargo install cargo-deny cargo-audit
git config core.hooksPath .githooks
```

Then verify the gates:

```bash
pre-commit run --all-files
pre-commit run --all-files --hook-stage pre-push
```

The hook stack is documented in
[docs/pre-commit-hooks.md](docs/pre-commit-hooks.md).

The product and engineering direction is documented in
[docs/DEVELOPMENT-SPEC.md](docs/DEVELOPMENT-SPEC.md).
The ordered implementation roadmap is in
[docs/IMPLEMENTATION-PLAN.md](docs/IMPLEMENTATION-PLAN.md).
The native Windows hardware procedure is in
[docs/WINDOWS-LAB.md](docs/WINDOWS-LAB.md), and evidence-backed support claims
are tracked in [docs/SUPPORT-MATRIX.md](docs/SUPPORT-MATRIX.md).
## Diagnostic CLI status

The current `s2bt` CLI exercises a deterministic fake backend only. It does not
access, discover, pair, connect to, or send commands to physical hardware.
Machine-readable output includes `"backend": "fake"` so simulated observations
cannot be confused with hardware evidence.

```bash
cargo run --bin s2bt -- scan
cargo run --bin s2bt -- --json scan
cargo run --bin s2bt -- pair fake-bee-021
```

On Windows, the read-only USB inventory command is the first real hardware
backend. It lists sanitized HID metadata for the BEE-021 controller and does not
open a device handle or expose any write/output operation:

```powershell
cargo run --bin s2bt -- --json usb-inventory
```

## Security

Please do not open public issues for vulnerabilities. Use GitHub private
vulnerability reporting for this repository, or contact the maintainer directly
if that is unavailable. See [SECURITY.md](SECURITY.md).

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
