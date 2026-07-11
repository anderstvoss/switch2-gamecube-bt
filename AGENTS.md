# Agent Instructions

Preserve this repository's security posture when making changes.

- Keep `unsafe_code = "forbid"` unless the maintainer explicitly changes the
  policy.
- Do not commit real `.env` files, secrets, credentials, logs, binary dumps, or
  local machine paths.
- Keep workflow actions SHA-pinned.
- Run the local pre-commit and pre-push checks before proposing a release or
  publish step.
- Prefer small, reviewed changes over broad refactors.
