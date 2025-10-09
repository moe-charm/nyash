#!/usr/bin/env bash
set -euo pipefail

# Legacy pattern sweep (non-destructive). Exits non-zero when issues found unless LEGACY_SWEEP_ALLOW=1.

ROOT_DIR=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT_DIR"

issues=0

echo "[sweep] scanning for legacy 'Key not found:' string-check anti-pattern ..."
if rg -n "Key not found:" --glob '!docs/**' --glob '!archive/**' >/tmp/legacy_key_not_found_hits.txt 2>/dev/null; then
  echo "[sweep] FOUND (non-docs):" >&2
  sed -n '1,200p' /tmp/legacy_key_not_found_hits.txt >&2 || true
  issues=$((issues+1))
else
  echo "[sweep] OK: no occurrences outside docs/archive." >&2
fi

echo "[sweep] scanning for '.length()' on collections (prefer .size()) ..."
if rg -n "\.length\(\)" --glob 'apps/**/*.hako' --glob 'examples/**/*.hako' >/tmp/legacy_length_hits.txt 2>/dev/null; then
  echo "[sweep] FOUND in apps/examples:" >&2
  sed -n '1,200p' /tmp/legacy_length_hits.txt >&2 || true
  issues=$((issues+1))
else
  echo "[sweep] OK: no occurrences in apps/examples." >&2
fi

if [[ "${LEGACY_SWEEP_ALLOW:-0}" != "1" && $issues -ne 0 ]]; then
  echo "[sweep] FAIL: legacy patterns detected ($issues)." >&2
  exit 2
fi

echo "[sweep] PASS: no legacy patterns detected."
exit 0

