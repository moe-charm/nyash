#!/bin/bash
# comparison_ops.sh - 比較演算（整数）の基本確認

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_comparisons() {
  local script='
// 期待: OK
if 3 > 2 {
  if 2 < 3 {
    print("OK")
  } else {
    print("NG")
  }
} else {
  print("NG")
}
'
  local output
  output=$(run_nyash_vm -c "$script" 2>&1)
  check_exact "OK" "$output" "comparison_basic"
}

run_test "comparison_basic" test_comparisons

