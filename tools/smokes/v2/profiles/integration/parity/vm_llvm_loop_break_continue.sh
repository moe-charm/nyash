#!/bin/bash
# vm_llvm_loop_break_continue.sh - VM vs LLVM parity for loop + break/continue

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_vm_llvm_loop_break_continue() {
  local code='local i, sum
i = 0
sum = 0
loop(i < 10) {
  i = i + 1
  if i % 2 == 0 { continue }
  sum = sum + i
  if i >= 7 { break }
}
print(sum)'
  check_parity -c "$code" "vm_llvm_loop_break_continue"
}

run_test "vm_llvm_loop_break_continue" test_vm_llvm_loop_break_continue

