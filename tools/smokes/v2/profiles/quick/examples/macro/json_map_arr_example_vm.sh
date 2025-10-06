#!/bin/bash
# json_map_arr_example_vm.sh — tiny example using map/arr sugar

source "$(dirname "$0")/../../../../lib/test_runner.sh"

require_env || exit 2

test_json_map_arr_example() {
  # Run the example file with macro paths enabled; expect no resolver/parse errors
  out=$(NYASH_MACRO_SELFHOST_MIN=1 \
        NYASH_MACRO_PATHS=apps/macros/selfhost_min/macros.hako \
        NYASH_SYNTAX_SUGAR_LEVEL=full \
        run_nyash_vm apps/examples/macro_sugar/mini_map_arr.hako --dev 2>&1 | filter_noise)
  if echo "$out" | grep -q "Unresolved function\|Parse error"; then
    test_fail "json_map_arr_example_vm" "resolver error"
  else
    test_pass "json_map_arr_example_vm"
  fi
}

run_test "json_map_arr_example_vm" test_json_map_arr_example
