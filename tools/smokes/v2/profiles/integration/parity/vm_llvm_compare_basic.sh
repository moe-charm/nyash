#!/bin/bash
# vm_llvm_compare_basic.sh - VM vs LLVM parity for basic integer comparisons

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_vm_llvm_compare_basic() {
  local code='if 3 > 2 {
  if 2 < 3 {
    print("OK")
  } else {
    print("NG")
  }
} else {
  print("NG")
}'
  check_parity -c "$code" "vm_llvm_compare_basic"
}

run_test "vm_llvm_compare_basic" test_vm_llvm_compare_basic

