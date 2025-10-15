#!/bin/bash
# else_if_nested_assign_vm.sh - nested else-if assigns only on one side; PHI must pick assigned value

source "$(dirname "$0")/../../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_else_if_nested_assign() {
  # Scenario:
  # - Top-level if is false; else branch contains nested if/else-if that assigns the variable.
  # - Only the else-if path assigns; the then-path (top-level) does not execute.
  # Expect PHI to merge with the assigned value from the nested branch (not the pre-if default).
  local script='
  local op, a, b, result
  op = "Sub"
  a = 45
  b = 3
  result = 0
  if op == "Add" {
  } else if op == "Sub" {
    result = a - b
  } else {
  }
  print(result)
  '
  local output
  output=$(run_nyash_vm -c "$script" 2>&1 | grep -v '^Result: ')
  check_exact "42" "$output" "else_if_nested_assign_vm" || return 1
  return 0
}

run_test "else_if_nested_assign_vm" test_else_if_nested_assign

