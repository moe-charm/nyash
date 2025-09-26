#!/bin/bash
# division_by_zero.sh - ゼロ除算エラーパターンの検証

source "$(dirname "$0")/../../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_division_by_zero() {
  local output
  output=$(run_nyash_vm -c 'print(1 / 0)' 2>&1 || true)
  check_regex "Division by zero" "$output" "division_by_zero"
}

run_test "division_by_zero" test_division_by_zero

