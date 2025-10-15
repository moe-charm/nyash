#!/bin/bash
# call_macro_array_get_vm.sh — call("Array.get/N", arr, idx) normalization

source "$(dirname "$0")/../../../lib/test_runner.sh"

require_env || exit 2

test_call_macro_array_get() {
  local code=$(cat << 'NYCODE'
static box Main {
  main() {
    local a = ["x", "y", "z"]
    // Normalize to ModuleFunction("ArrayBox.get/1", [a, 1]) and run without resolver errors
    local v = call("Array.get/2", a, 1)
    return 0
  }
}
NYCODE
)
  out=$(NYASH_MACRO_SELFHOST_MIN=1         NYASH_MACRO_PATHS=apps/macros/selfhost_min/macros.hako         NYASH_SYNTAX_SUGAR_LEVEL=full         run_nyash_vm -c "$code" --dev 2>&1 | filter_noise)
  if echo "$out" | grep -q "Unresolved function\|Parse error"; then
    test_fail "call_macro_array_get_vm" "resolver error"
  else
    test_pass "call_macro_array_get_vm"
  fi
}

run_test "call_macro_array_get_vm" test_call_macro_array_get
