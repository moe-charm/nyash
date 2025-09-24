#!/bin/bash
# early_return.sh - 早期returnの合流確認

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_early_return_then() {
  local script='
static box Main {
  main() {
    if 1 {
      print("A")
      return 0
    } else {
      print("B")
    }
    print("C") // 到達しない
  }
}
'
  local output
  output=$(run_nyash_vm -c "$script" 2>&1)
  check_exact "A" "$output" "early_return_then"
}

run_test "early_return_then" test_early_return_then

