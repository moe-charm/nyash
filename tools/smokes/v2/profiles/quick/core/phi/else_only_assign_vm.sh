#!/bin/bash
# else_only_assign_vm.sh - Only else-branch assigns; PHI must pick else value

source "$(dirname "$0")/../../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_else_only_assign() {
  local script='
  local result
  result = 0
  if 0 { /* no assign */ } else { result = 42 }
  print(result)
  '
  local output
  output=$(run_nyash_vm -c "$script" 2>&1 | grep -v '^Result: ')
  check_exact "42" "$output" "else_only_assign_vm" || return 1
  return 0
}

run_test "else_only_assign_vm" test_else_only_assign

