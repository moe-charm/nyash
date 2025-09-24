#!/bin/bash
# vm_llvm_loop_sum.sh - VM vs LLVM parity for simple loop accumulation

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_vm_llvm_loop_sum() {
  local code='local i, sum
i = 1
sum = 0
while i <= 5 {
  sum = sum + i
  i = i + 1
}
print(sum)'
  check_parity -c "$code" "vm_llvm_loop_sum"
}

run_test "vm_llvm_loop_sum" test_vm_llvm_loop_sum

