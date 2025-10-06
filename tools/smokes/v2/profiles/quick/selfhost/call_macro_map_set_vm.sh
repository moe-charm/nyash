#!/bin/bash
# call_macro_map_set_vm.sh — call("Map.set/3", map, key, val) normalization

source "$(dirname "$0")/../../../lib/test_runner.sh"

require_env || exit 2

test_call_macro_map_set() {
  local code=$(cat << 'NYCODE'
static box Main {
  main() {
    local m = map({})
    // Normalize to ModuleFunction("MapBox.set/2", [m, "k", 1]) and run without resolver errors
    call("Map.set/3", m, "k", 1)
    return 0
  }
}
NYCODE
)
  out=$(NYASH_MACRO_SELFHOST_MIN=1 \
        NYASH_MACRO_PATHS=apps/macros/selfhost_min/macros.hako \
        NYASH_SYNTAX_SUGAR_LEVEL=full \
        run_nyash_vm -c "$code" --dev 2>&1 | filter_noise)
  if echo "$out" | grep -q "Unresolved function\|Parse error"; then
    test_fail "call_macro_map_set_vm" "resolver error"
  else
    test_pass "call_macro_map_set_vm"
  fi
}

run_test "call_macro_map_set_vm" test_call_macro_map_set
