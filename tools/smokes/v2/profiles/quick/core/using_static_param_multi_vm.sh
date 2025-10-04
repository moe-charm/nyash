#!/bin/bash
# using_static_param_multi_vm.sh — Ensure using→static box call preserves multiple parameters

source "$(dirname "$0")/../../../lib/test_runner.sh"
export NYASH_ALLOW_USING_FILE=1
require_env || exit 2
preflight_plugins || exit 2

if [ "${SMOKES_ENABLE_ROOTFIX:-0}" != "1" ]; then
  test_skip "using_static_param_multi_vm (root-fix WIP)" "Enable with SMOKES_ENABLE_ROOTFIX=1" || exit 0
  exit 0
fi

TMP_DIR="/tmp/using_static_param_multi_vm_$$"
mkdir -p "$TMP_DIR"

# Create a static box with multi-arg echo helpers
cat > "$TMP_DIR/echo2_box.hako" << 'H'
static box Echo2Box {
  // Return s.length() + n
  sum_len(s, n) { return s.length() + n }
  // Return first 3 chars (ASCII-safe)
  head3(s) { return s.substring(0, 3) }
}
H

cat > "$TMP_DIR/driver.nyash" << 'NY'
using "/tmp/USING_PLACEHOLDER/echo2_box.hako" as Echo2

static box Main {
  main() {
    local s = "abcdef"
    local n = 4
    local sum = Echo2.sum_len(s, n)   // expect 6 + 4 = 10
    local h = Echo2.head3(s)          // expect "abc"
    print("SUM="+(""+sum))
    print("HEAD="+h)
    return 0
  }
}
NY

# Replace placeholder path
sed -i "s|/tmp/USING_PLACEHOLDER|$TMP_DIR|g" "$TMP_DIR/driver.nyash"

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tr -d '\r' | tail -n 2 | xargs echo)
expected="SUM=10 HEAD=abc"

test_name="using_static_param_multi_vm"
compare_outputs "$expected" "$out" "$test_name" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

