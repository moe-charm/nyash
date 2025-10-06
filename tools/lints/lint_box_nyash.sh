#!/bin/bash
# lint_box_nyash.sh — Detect non-archived Box modules still using .nyash extension
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cd "$ROOT_DIR"

echo "[lint] scanning for .nyash under apps/**/boxes (excluding archives)" >&2
mapfile -t hits < <(find apps -type f -path '*/boxes/*.nyash' \
  -not -path 'apps/archive/*' -not -path 'apps/*legacy*/*' -print)

if [ ${#hits[@]} -eq 0 ]; then
  echo "[lint] OK: no .nyash boxes found outside archives" >&2
  exit 0
fi

echo "[lint] Found .nyash boxes (consider migrating to .hako):" >&2
for f in "${hits[@]}"; do
  echo "  $f"
done

if [ "${LINT_BOX_NYASH_FAIL:-0}" = "1" ]; then
  echo "[lint] Failing due to LINT_BOX_NYASH_FAIL=1" >&2
  exit 1
fi

exit 0
