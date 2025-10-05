#!/usr/bin/env bash
set -euo pipefail

# DEV lint: detect 2-arg String.indexOf or function-like indexOf(a,b)
# Goal: migrate to index_of_from(text, needle, pos) helpers (CfgNavigatorBox/StringScanBox/JsonCursorBox).

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT_DIR"

exclude=(
  "--glob" "!target/**"
  "--glob" "!artifacts/**"
  "--glob" "!tools/smokes/**"
  "--glob" "!archive/**"
)

matches=$(rg -n "\.indexOf\([^)]*,[^)]*\)" -S "${exclude[@]}" || true)

if [[ -z "$matches" ]]; then
  echo "✅ DEV-LINT indexOf(2): no matches"
  exit 0
fi

echo "⚠️ DEV-LINT indexOf(2): potential 2-arg indexOf usages found"
echo "$matches" | sed 's/^/  - /'
echo

echo "Hint: replace with index_of_from(text, needle, pos) from CfgNavigatorBox/StringScanBox/JsonCursorBox."

if [[ "${LINT_INDEXOF_FAIL:-0}" == "1" ]]; then
  exit 1
fi
exit 0
