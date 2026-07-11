# switch2-gamecube-bt

Rust workspace for Switch 2 GameCube Bluetooth experiments.

The repository includes pinned local checks, committed git hook wrappers,
secret and artifact blockers, Rust lint/test gates, supply-chain checks,
CodeQL, OpenSSF Scorecard, SBOM generation, Dependabot, issue/PR templates, and
security reporting documentation.

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

## Security

Please do not open public issues for vulnerabilities. Use GitHub private
vulnerability reporting for this repository, or contact the maintainer directly
if that is unavailable. See [SECURITY.md](SECURITY.md).

## License

AGPL-3.0-only. See [LICENSE](LICENSE).
