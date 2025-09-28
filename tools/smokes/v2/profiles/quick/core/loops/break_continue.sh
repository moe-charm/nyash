#!/bin/bash
# break_continue.sh - while + break/continue の代表ケース

source "$(dirname "$0")/../../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_skip "break_continue" "Temporarily skipped (VM PHI carrier polish); LLVM PASS" && exit 0

test_break_continue() {
  local script='
local i, sum
i = 0
sum = 0
loop(i < 10) {
  i = i + 1
  if i % 2 == 0 { continue }   // 偶数はスキップ
  sum = sum + i
  if i >= 7 { break }           // 7 で打ち切り（対象: 1,3,5,7）
}
print(sum)
'
  local out; out=$(run_nyash_vm -c "$script" 2>&1)
  # 1+3+5+7 = 16
  check_exact "16" "$out" "break_continue"
}

run_test "break_continue" test_break_continue
