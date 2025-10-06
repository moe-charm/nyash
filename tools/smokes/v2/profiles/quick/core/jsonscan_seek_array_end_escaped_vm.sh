#!/bin/bash
# jsonscan_seek_array_end_escaped_vm.sh — Verify JsonScanBox.seek_array_end with escaped brackets in strings

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

if [ "${SMOKES_ENABLE_ROOTFIX:-0}" != "1" ]; then
  test_skip "jsonscan_seek_array_end_escaped_vm (root-fix WIP)" "Enable with SMOKES_ENABLE_ROOTFIX=1" || exit 0
  exit 0
fi

TMP_DIR="/tmp/jsonscan_seek_array_end_escaped_vm_$$"
mkdir -p "$TMP_DIR"

export NYASH_USING=1
export NYASH_MODULES="json.scan=apps/selfhost/common/json/core/json_scan.hako"

cat > "$TMP_DIR/driver.nyash" << 'NY'
using json.scan as JsonScanBox

static box Main {
  main() {
    // Array containing a string with a closing bracket character
    local s = "[\"value with ] inside\",{}]"
    // '[' is at index 0
    local e = JsonScanBox.seek_array_end(s, 0)
    print("E="+(""+e))
    return 0
  }
}
NY

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tr -d '\r' | tail -n 1 | xargs echo)
# Expect: last closing bracket index is length-1 (here 27)
expected="E=25"

test_name="jsonscan_seek_array_end_escaped_vm"
compare_outputs "$expected" "$out" "$test_name" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
