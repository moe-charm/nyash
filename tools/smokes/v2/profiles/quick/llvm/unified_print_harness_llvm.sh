#!/usr/bin/env bash
# unified_print_harness_llvm.sh — Verify that Unified Call prints via harness (nyash.console.log mapping)

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$ROOT_DIR"
while [ ! -f "$ROOT/Cargo.toml" ] && [ "$ROOT" != "/" ]; do
  ROOT="$(dirname "$ROOT")"
done

APP="$ROOT/apps/tests/unified_print_simple.nyash"
OUT="$ROOT/tmp/unified_print_harness"

if [ ! -f "$APP" ]; then
  echo 'print("hello_unified_print")' > "$APP"
  echo 'return 0' >> "$APP"
fi

# Build via harness → link → run
NYASH_MIR_UNIFIED_CALL=1 NYASH_LLVM_USE_HARNESS=1 NYASH_NYRT_SILENT_RESULT=1 "$ROOT"/tools/build_llvm.sh "$APP" -o "$OUT" >/dev/null || true

out=$("$OUT" 2>&1 || true)
if echo "$out" | grep -q '^hello_unified_print$'; then
  echo "OK: Unified print mapped via harness"
  exit 0
else
  echo "ERROR: unified print missing; got:" >&2
  echo "$out" >&2
  exit 1
fi
