#!/bin/bash
# vm_llvm_branch_loop_min.sh - VM vs LLVM parity for branch+loop minimal case

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2
preflight_plugins || exit 2

test_vm_llvm_branch_loop_min() {
  local code='local i, sum
i = 0
sum = 0
while i < 5 {
  i = i + 1
  if (i % 2) == 0 { continue }
  sum = sum + i
}
print(sum)'
  check_parity -c "$code" "vm_llvm_branch_loop_min"
}

run_test "vm_llvm_branch_loop_min" test_vm_llvm_branch_loop_min

