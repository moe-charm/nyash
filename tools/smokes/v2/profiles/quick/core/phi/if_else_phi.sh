#!/bin/bash
# if_else_phi.sh - PHI wiring: else-if chain should pick exit predecessors

source "$(dirname "$0")/../../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_if_else_phi() {
  local script='
  local score, x
  score = 75
  if score >= 80 {
    x = "A"
  } else if score >= 60 {
    x = "B"
  } else {
    x = "C"
  }
  print(x)
  '
  local output
  output=$(run_nyash_vm -c "$script" 2>&1)
  check_exact "B" "$output" "if_else_phi"
}

run_test "if_else_phi" test_if_else_phi

