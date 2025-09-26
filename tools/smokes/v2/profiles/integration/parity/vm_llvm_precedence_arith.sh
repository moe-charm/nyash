#!/bin/bash
# vm_llvm_precedence_arith.sh - arithmetic precedence parity (1 + 2 * 3 == 7, (1+2)*3==9)

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_vm_llvm_precedence_arith() {
  local code='print(1 + 2 * 3)\nprint((1 + 2) * 3)'
  check_parity -c "$code" "vm_llvm_precedence_arith"
}

run_test "vm_llvm_precedence_arith" test_vm_llvm_precedence_arith

