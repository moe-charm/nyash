#!/bin/bash
# using_static_param_vm.sh — Ensure using→static box call preserves parameters

source "$(dirname "$0")/../../../lib/test_runner.sh"
export NYASH_ALLOW_USING_FILE=1
require_env || exit 2
preflight_plugins || exit 2

if [ "${SMOKES_ENABLE_ROOTFIX:-0}" != "1" ]; then
  test_skip "using_static_param_vm (root-fix WIP)" "Enable with SMOKES_ENABLE_ROOTFIX=1" || exit 0
  exit 0
fi

TMP_DIR="/tmp/using_static_param_vm_$$"
mkdir -p "$TMP_DIR"

# Create a tiny static box to echo back parameter length and head
cat > "$TMP_DIR/echo_box.hako" << 'H'
static box EchoBox {
  echo_len(s) {
    // return string length as int
    return s.length()
  }
  echo_head3(s) {
    // return first 3 chars as string
    return s.substring(0, 3)
  }
}
H

cat > "$TMP_DIR/driver.nyash" << 'NY'
using "/tmp/USING_PLACEHOLDER/echo_box.hako" as Echo

static box Main {
  main() {
    local s = "abcdef"
    local n = Echo.echo_len(s)
    local h = Echo.echo_head3(s)
    print("LEN="+(""+n))
    print("HEAD="+h)
    return 0
  }
}
NY

# Replace placeholder path
sed -i "s|/tmp/USING_PLACEHOLDER|$TMP_DIR|g" "$TMP_DIR/driver.nyash"

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tr -d '' | tail -n 2 | xargs echo)
expected="LEN=6 HEAD=abc"

test_name="using_static_param_vm"
compare_outputs "$expected" "$out" "$test_name" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
