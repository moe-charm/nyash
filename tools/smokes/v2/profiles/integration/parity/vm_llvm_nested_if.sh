#!/bin/bash
# vm_llvm_nested_if.sh - VM vs LLVM parity for nested if with merge PHI

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_vm_llvm_nested_if() {
  local code='local a, b
a = 10
b = 20
if a < b {
  if a == 10 {
    print("correct")
  } else {
    print("wrong")
  }
} else {
  print("error")
}'
  check_parity -c "$code" "vm_llvm_nested_if"
}

run_test "vm_llvm_nested_if" test_vm_llvm_nested_if

