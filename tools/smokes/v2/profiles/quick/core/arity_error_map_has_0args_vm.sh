#!/bin/bash
# arity_error_map_has_0args_vm.sh — Map.has with 0 args should error

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/arity_error_map_has_0args_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NYEOF'
static box Main {
  main() {
    local m = new MapBox()
    // Wrong arity: has expects 1 arg
    if m.has() { return 101 }
    return 0
  }
}
NYEOF

if run_nyash_vm "$TMP_DIR/driver.nyash" >/dev/null 2>&1; then
  echo "FAIL: expected non-zero exit" >&2
  rm -rf "$TMP_DIR"; exit 1
else
  echo "OK"
  rm -rf "$TMP_DIR"; exit 0
fi
