#!/usr/bin/env bash
# CI guard: forbid direct core box construction outside approved locations
# Fails if it finds "ArrayBox::new(" or "MapBox::new(" or "StringBox::new(" in disallowed files.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT_DIR"

patterns=(
  'ArrayBox::new\('
  'MapBox::new\('
  'StringBox::new\('
)

# Allowlist (regex for file paths):
# - Type registry (central factory)
# - Box implementations that legitimately construct internals
# - Plugins and tests/examples/tools (development code)
allow_re='^(src/runtime/type_registry\.rs|src/boxes/[^/]+/.*|plugins/|tests/|examples/|tools/|src/backend/.*/extern_adapter\.rs)'

fail=0
for pat in "${patterns[@]}"; do
  while IFS= read -r -d '' file; do
    if [[ "$file" =~ $allow_re ]]; then
      continue
    fi
    if rg -n --pcre2 "$pat" "$file" > /dev/null; then
      echo "[CI-guard] Forbidden core box construction in: $file (pattern: $pat)" >&2
      rg -n --pcre2 "$pat" "$file" || true
      fail=1
    fi
  done < <(rg -l --pcre2 "$pat" | tr '\n' '\0')
done

if [ "$fail" -ne 0 ]; then
  echo "[CI-guard] FAIL: direct core box construction detected outside registry/approved files" >&2
  exit 1
fi

echo "[CI-guard] PASS: no forbidden core box construction detected"
exit 0
