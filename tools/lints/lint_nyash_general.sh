#!/bin/bash
# lint_nyash_general.sh — Detect .nyash files outside allowed areas (Stage-0/VM/Smokes)
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

echo "[lint] scanning for .nyash outside allowed areas" >&2

mapfile -t hits < <(find apps -type f -name '*.nyash'   -not -path 'apps/selfhost/ny-parser-nyash/*'   -not -path 'apps/selfhost/parser/*'   -not -path 'apps/selfhost/vm/*'   -not -path 'apps/selfhost/smokes/*'   -not -path 'apps/tests/*'   -not -path 'examples/*'   -not -path 'archive/*'   -print)

if [ ${#hits[@]} -eq 0 ]; then
  echo "[lint] OK: no stray .nyash files found" >&2
  exit 0
fi

echo "[lint] Found stray .nyash files (migrate to .hako):" >&2
for f in "${hits[@]}"; do echo "  $f"; done

if [ "${LINT_NYASH_GENERAL_FAIL:-0}" = "1" ]; then
  echo "[lint] Failing due to LINT_NYASH_GENERAL_FAIL=1" >&2
  exit 1
fi

exit 0
