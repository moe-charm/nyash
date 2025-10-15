#!/usr/bin/env bash
# unified_recursion_method_llvm.sh — Unified Call recursion (instance Method)

set -euo pipefail

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

if [[ "${SMOKES_ENABLE_UNIFIED_RECURSION:-}" != "1" ]]; then
  test_skip "Unified recursion (Method) gated; set SMOKES_ENABLE_UNIFIED_RECURSION=1"
  exit 0
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$ROOT_DIR"
while [ ! -f "$ROOT/Cargo.toml" ] && [ "$ROOT" != "/" ]; do
  ROOT="$(dirname "$ROOT")"
done

TMP_DIR="/tmp/unified_recursion_method_llvm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR"/driver.nyash << 'EOAPP'
box Fibber {
  birth() { }
  fib(n) {
    if (n < 2) { return n }
    return me.fib(n - 1) + me.fib(n - 2)
  }
}

static box Main {
  main() {
    local f = new Fibber()
    local v = f.fib(10)
    if (v == 55) { print("ok") } else { print("ng") }
    return 0
  }
}
EOAPP

OUT="/tmp/unified_recursion_method_bin_$$"
NYASH_LLVM_USE_HARNESS=1 NYASH_MIR_UNIFIED_CALL=1 NYASH_NYRT_SILENT_RESULT=1 pushd "$ROOT" >/dev/null
NYASH_LLVM_USE_HARNESS=1 NYASH_MIR_UNIFIED_CALL=1 NYASH_NYRT_SILENT_RESULT=1 ./tools/build_llvm.sh "$TMP_DIR/driver.nyash" -o "$OUT"
popd >/dev/null >/dev/null || true
out=$("$OUT" 2>&1 || true)

exp="ok"
act=$(echo "$out" | tail -n 1 | tr -d '\r' | xargs || true)
compare_outputs "$exp" "$act" "unified_recursion_method_llvm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

cd /
rm -rf "$TMP_DIR"
exit 0
