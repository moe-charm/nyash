#!/bin/bash
# jsonscan_seek_array_end_vm.sh — Verify JsonScanBox.seek_array_end on minimal arrays

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

if [ "${SMOKES_ENABLE_ROOTFIX:-0}" != "1" ]; then
  test_skip "jsonscan_seek_array_end_vm (root-fix WIP)" "Enable with SMOKES_ENABLE_ROOTFIX=1" || exit 0
  exit 0
fi

TMP_DIR="/tmp/jsonscan_seek_array_end_vm_$$"
mkdir -p "$TMP_DIR"

export NYASH_USING=1
export NYASH_MODULES="json.scan=selfhost/shared/json/core/json_scan.hako"

cat > "$TMP_DIR/driver.nyash" << 'NY'
using json.scan as JsonScanBox

static box Main {
  main() {
    local s = "[{}]"
    local e = JsonScanBox.seek_array_end(s, 0)
    print("E="+(""+e))
    // nested
    local t = "[[{}]]"
    local e2 = JsonScanBox.seek_array_end(t, 0)
    print("E2="+(""+e2))
    return 0
  }
}
NY

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tr -d '' | tail -n 2 | xargs echo)
# Expect: E=3 (for "[{}]") and E2=5 (index of last ']' in t)
expected="E=3 E2=5"

test_name="jsonscan_seek_array_end_vm"
compare_outputs "$expected" "$out" "$test_name" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
