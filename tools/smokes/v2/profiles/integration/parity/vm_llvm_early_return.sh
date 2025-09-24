#!/bin/bash
# vm_llvm_early_return.sh - VM vs LLVM parity for early return

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_vm_llvm_early_return() {
  local code='static box Main {
  main() {
    if 1 {
      print("A")
      return 0
    } else {
      print("B")
    }
    print("C")
  }
}'
  check_parity -c "$code" "vm_llvm_early_return"
}

run_test "vm_llvm_early_return" test_vm_llvm_early_return

