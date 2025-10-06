#!/bin/bash
# call_macro_string_indexof_vm.sh — call("String.indexOf/N", s, needle) normalization

source "$(dirname "$0")/../../../lib/test_runner.sh"

require_env || exit 2

test_call_macro_string_indexof() {
  local code=$(cat << 'NYCODE'
static box Main {
  main() {
    local s = "abc"
    // Normalize to ModuleFunction("StringBox.indexOf/1", [s, "b"]) and run without resolver errors
    local n = call("String.indexOf/2", s, "b")
    return 0
  }
}
NYCODE
)
  out=$(NYASH_MACRO_SELFHOST_MIN=1         NYASH_MACRO_PATHS=apps/macros/selfhost_min/macros.hako         NYASH_SYNTAX_SUGAR_LEVEL=full         run_nyash_vm -c "$code" --dev 2>&1 | filter_noise)
  if echo "$out" | grep -q "Unresolved function\|Parse error"; then
    test_fail "call_macro_string_indexof_vm" "resolver error"
  else
    test_pass "call_macro_string_indexof_vm"
  fi
}

run_test "call_macro_string_indexof_vm" test_call_macro_string_indexof
