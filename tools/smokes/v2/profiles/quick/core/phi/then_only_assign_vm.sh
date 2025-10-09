#!/bin/bash
# then_only_assign_vm.sh - Only then-branch assigns; PHI must pick then value

source "$(dirname "$0")/../../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_then_only_assign() {
  local script='
  local result
  result = 0
  if 1 { result = 42 } else { }
  print(result)
  '
  local output
  output=$(run_nyash_vm -c "$script" 2>&1 | grep -v '^Result: ')
  check_exact "42" "$output" "then_only_assign_vm" || return 1
  return 0
}

run_test "then_only_assign_vm" test_then_only_assign

