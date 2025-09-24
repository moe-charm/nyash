#!/bin/bash
# var_merge_delta.sh - then/else で同一変数を更新 → merge 時に正しい値を選択

source "$(dirname "$0")/../../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_var_merge_delta() {
  local script='
  local a
  a = 1
  if 1 {
    a = 2
  } else {
    a = 3
  }
  print(a)
  '
  local output
  output=$(run_nyash_vm -c "$script" 2>&1)
  check_exact "2" "$output" "var_merge_delta"
}

run_test "var_merge_delta" test_var_merge_delta

