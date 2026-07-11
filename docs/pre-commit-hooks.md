# Pre-commit hooks — developer guide

This document covers installation, what each hook guards, how to verify
the hooks are active, what to do when a hook blocks you, and how to run
them manually. Read the public `CONTRIBUTING.md` for the general PR
workflow.

---

## One-time setup

```bash
# 1. Install pre-commit system-wide
pipx install pre-commit          # preferred
# or: pip install --user pre-commit

# 2. Install the Cargo security tools
cargo install cargo-deny cargo-audit

# 3. Point Git at the committed hook wrappers
git config core.hooksPath .githooks

# 4. Remove any stale wrappers left over from a prior `pre-commit install`
rm -f .git/hooks/pre-commit .git/hooks/pre-push
```

The committed wrappers in `.githooks/` call whichever `pre-commit` is on
`PATH`, so you never need to re-run `pre-commit install` after a rebase or
re-clone.

### Verify the setup

```bash
git config core.hooksPath           # → .githooks
ls .githooks/                       # → pre-commit  pre-push (both executable)
pre-commit run --all-files          # all commit-stage hooks should pass
pre-commit run --all-files \
  --hook-stage pre-push             # cargo-deny + cargo-audit should pass
```

---

## Hook inventory

### Commit-stage (`pre-commit`)

All of these run on every `git commit`. They must all pass before the
commit is recorded.

| Hook | What it catches |
|---|---|
| `gitleaks-system` | Credentials with entropy analysis: API keys, tokens, private keys, connection strings. Uses a gitleaks binary on `PATH` and its v8 rules. |
| `detect-private-key` | `BEGIN … PRIVATE KEY` headers in any text file. |
| `check-yaml / toml / json` | Syntax errors in config files. |
| `end-of-file-fixer` | Missing trailing newline. |
| `trailing-whitespace` | Trailing spaces/tabs on any line. |
| `check-merge-conflict` | Unresolved conflict markers (`<<<<<<<`). |
| `check-added-large-files` | Files over 500 KB. |
| `block-env-files` | Any `.env` file (except `.env.example`). |
| `block-private-key-files` | Files whose **name** contains `id_rsa`, `id_ed25519`, `id_ecdsa`, `id_dsa`, `.pem`, `.key`, `.p12`, `credential(s)`, or `secret(s)`. |
| `block-logs` | Files with `.log`, `.dump`, or `.trace` extensions. |
| `block-local-paths` | Hardcoded machine paths: Linux home dirs, macOS home dirs, Windows/WSL user dirs, UNC shares. |
| `block-ssh-hosts` | SSH config directives (HostName, IdentityFile), OpenSSH private key headers, GitHub PAT token prefixes (fine-grained and classic). |
| `block-cloud-uris` | Private cloud storage URIs: S3, GCS, Azure Blob, Alibaba OSS, and WASB/ABFS variants. |
| `block-local-network-targets` | Loopback and RFC-1918 addresses: named loopback, IPv4/IPv6 loopback, any-address bind, class-A/B/C private ranges, Docker-internal hostnames. |
| `block-binary-artifacts` | `.sqlite`, `.db`, `.tar`, `.gz`, `.zip`, `.7z`, `.rar`, `.bin`, `.exe`, `.dll`, `.so`, `.dylib`. |
| `cargo fmt` | Code must be formatted (`cargo fmt --check`). |
| `cargo check` | Must compile cleanly. |
| `cargo clippy` | No warnings (`-D warnings`). |
| `cargo test` | Full test suite must pass. |

### Push-stage (`pre-push`)

These run on every `git push`, after the commit-stage hooks.

| Hook | What it catches |
|---|---|
| `gitleaks detect` | Full-tree scan of the working copy. |
| Tracked-file blocklist | `git ls-files` piped through the same credential/key/secret filename patterns as `block-private-key-files`. |
| Local-path grep | `git grep` across all tracked files for Linux, macOS, Windows, and WSL user directory paths. |
| `cargo deny` | License, advisory, and duplicate-crate policy. |
| `cargo audit` | Known CVEs in the dependency tree. |

### CI (GitHub Actions)

CI re-runs the cargo gate on the full matrix (Ubuntu + macOS + Windows)
plus `cargo-deny`, `cargo-audit`, dependency-review on PRs, and OpenSSF
Scorecard on pushes to `main`. A weekly scheduled workflow runs `gitleaks`
over the full commit history.

---

## What gitleaks does and does not catch

Gitleaks uses entropy analysis plus a library of per-service regex rules.
It catches real, high-entropy secrets. It does **not** catch:

- Synthetic test strings that match well-known documentation placeholders
  (published AWS example keys, Stripe test tokens, and similar). Gitleaks
  ships an internal allowlist of these values.

**Rule:** never use published example strings as test fixtures, even in
unit tests. Use random-looking but structurally valid fakes, and mark the
file with a comment like `# test fixture — not a real credential`.

---

## Common hook failures and fixes

### `gitleaks` / `detect-private-key` — private key block

You have a PEM or OpenSSH key in a staged file.

- **Remove it.** Keys must never be committed, even in test fixtures.
- If you genuinely need a test key, generate a throwaway one and store
  only the *public* half, or use a fixed test vector from a published RFC.

### `block-env-files` — `.env` committed

Commit `.env.example` (schema only, no real values) and load secrets from
the environment or a secrets manager at runtime.

### `block-local-paths` / `block-local-network-targets`

A hardcoded path or address slipped into source or documentation.

- Replace it with a config value, environment variable, or a placeholder
  like `<your-host>`.
- If the path or address is intentional documentation (e.g. explaining
  what to put in a config field), rewrite it as prose rather than a
  literal value.

### `block-ssh-hosts` — `HostName` or `IdentityFile` in a text file

Never commit SSH config fragments. Move host configuration to your local
`~/.ssh/config`.

### `block-cloud-uris` — `s3://`, `gs://`, etc.

Private bucket names are sensitive. Replace the literal URI with an
environment variable or config key:

```bash
# bad — literal cloud URI leaks a private resource name (blocked by hook)
BACKUP_PATH="<your-bucket-scheme-and-name-here>"

# good — read the URI from the environment at runtime
BACKUP_PATH="${BACKUP_BUCKET_URI}"
```

### `cargo fmt` failure

```bash
cargo fmt --all
git add -u
```

### `cargo clippy` failure

Fix the lint or — only if the lint is genuinely inapplicable — add a
targeted `#[allow(…)]` with a comment explaining why.

### `cargo test` failure

Fix the failing test before committing. Do not `--skip` or comment out
the test.

---

## Running hooks manually

```bash
# Run all commit-stage hooks against every tracked file
pre-commit run --all-files

# Run a single hook by id
pre-commit run gitleaks-system --all-files
pre-commit run block-local-paths --all-files

# Run only push-stage hooks (cargo-deny + cargo-audit)
pre-commit run --all-files --hook-stage pre-push

# Run against staged files only (mirrors what git commit does)
pre-commit run

# Simulate a push (reads ref info from stdin as git would supply it)
echo "refs/heads/main $(git rev-parse HEAD) refs/heads/main $(git rev-parse HEAD~1)" \
  | pre-commit run --hook-stage pre-push
```

---

## Adding or changing a hook

All hook configuration lives in `.pre-commit-config.yaml`. After any
change:

1. Run `pre-commit run --all-files` to confirm existing content still
   passes.
2. Add a test fixture under `tests/secret-detection/` that exercises the
   new pattern, stage it, run `pre-commit run --files
   tests/secret-detection/<file>`, confirm the hook fires, then clean up
   with `git restore --staged && rm`.
3. Update this document's hook inventory table.
4. If the hook also has a mirror in `.githooks/pre-push` (the tracked-file
   blocklist or local-path grep), update that script to match.
