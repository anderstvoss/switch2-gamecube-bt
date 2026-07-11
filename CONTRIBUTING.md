# Contributing

Thanks for helping improve `switch2-gamecube-bt`.

## Required Local Gates

Before opening a pull request, run:

```bash
pre-commit run --all-files
pre-commit run --all-files --hook-stage pre-push
```

The hook stack checks formatting, linting, tests, secret patterns, suspicious
local files, local machine paths, private network targets, dependency policy,
and known Rust advisories.

## Pull Requests

Keep changes focused, include tests for behavior changes, and update docs when
the developer workflow or security posture changes.

Security reports should go through private vulnerability reporting rather than
public issues.
