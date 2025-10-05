#!/usr/bin/env bash
# unified_print_variants_harness_llvm.sh — Check print/println/log normalization via harness

set -euo pipefail

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

if [[ "${SMOKES_ENABLE_PRINT_VARIANTS:-}" != "1" ]]; then
  test_skip "print variants gated; set SMOKES_ENABLE_PRINT_VARIANTS=1"
  exit 0
fi


ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$ROOT_DIR"
while [ ! -f "$ROOT/Cargo.toml" ] && [ "$ROOT" != "/" ]; do
  ROOT="$(dirname "$ROOT")"
done

TMP_DIR="/tmp/unified_print_variants_harness_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR"/driver.nyash << 'EOAPP'
print("hello_print")
println("hello_println")
log("hello_log")
return 0
EOAPP

OUT="/tmp/unified_print_variants_bin_$$"
pushd "$ROOT" >/dev/null
NYASH_LLVM_USE_HARNESS=1 NYASH_MIR_UNIFIED_CALL=1 NYASH_NYRT_SILENT_RESULT=1 ./tools/build_llvm.sh "$TMP_DIR/driver.nyash" -o "$OUT" >/dev/null || true
popd >/dev/null
out=$("$OUT" 2>&1 || true)

ok=1
for s in hello_print hello_println hello_log; do
  if ! echo "$out" | grep -qx "$s"; then
    ok=0
    break
  fi
done

if [[ $ok -eq 1 ]]; then
  echo "OK: print/println/log normalized via harness"
  cd /; rm -rf "$TMP_DIR"; exit 0
else
  echo "ERROR: missing normalized output; got:" >&2
  echo "$out" >&2
  cd /; rm -rf "$TMP_DIR"; exit 1
fi
