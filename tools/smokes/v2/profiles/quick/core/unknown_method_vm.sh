#!/bin/bash
# unknown_method_vm.sh — calling unknown method should error (rc-only)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/unknown_method_vm_$$"
mkdir -p "$TMP_DIR"
cat > "$TMP_DIR/driver.nyash" << 'NY'
static box Main {
  main() {
    local a = new ArrayBox()
    // unknown method name
    a.frobnicate(1)
    return 0
  }
}
NY
if run_nyash_vm "$TMP_DIR/driver.nyash" >/dev/null 2>&1; then
  echo "FAIL: expected non-zero rc" >&2
  rm -rf "$TMP_DIR"; exit 1
else
  echo "OK"
  rm -rf "$TMP_DIR"; exit 0
fi
