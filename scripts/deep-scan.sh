#!/usr/bin/env bash
# scripts/deep-scan.sh
#
# Operator helper: run gitleaks across the entire commit history of
# every local ref (branches and tags), not just the current HEAD's
# history. Use this before publishing a sync or merging anything that
# matters — the every-run pre-commit and pre-push hooks scan staged
# content and the current ref's history only.
#
# Usage:
#   scripts/deep-scan.sh

set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

if ! command -v gitleaks >/dev/null 2>&1; then
  echo "ERROR: gitleaks not on PATH (install: https://github.com/gitleaks/gitleaks)" >&2
  exit 1
fi

echo "→ fetching all refs (branches + tags)"
git fetch --all --tags --prune --quiet || echo "  (fetch skipped — no remote / offline)"

echo "→ gitleaks detect --log-opts='--all'"
gitleaks detect --no-banner --log-opts='--all'

echo "✓ deep scan PASS"
