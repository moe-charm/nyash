#!/bin/bash
# if_without_else_phi.sh - PHI wiring: elseなしの変数マージ（false-edge→merge）

source "$(dirname "$0")/../../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_if_without_else_phi() {
  local script='
  local v
  v = "before"
  if 0 {
    v = "then"
  }
  print(v)
  '
  local output
  output=$(run_nyash_vm -c "$script" 2>&1)
  check_exact "before" "$output" "if_without_else_phi"
}

run_test "if_without_else_phi" test_if_without_else_phi

