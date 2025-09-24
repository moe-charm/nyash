#!/bin/bash
# vm_llvm_unary_not.sh - unary 'not' parity

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_vm_llvm_unary_not() {
  local code='print(not false)'
  check_parity -c "$code" "vm_llvm_unary_not"
}

run_test "vm_llvm_unary_not" test_vm_llvm_unary_not

