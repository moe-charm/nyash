#!/bin/bash
# vm_llvm_if_without_else_phi.sh - VM vs LLVM parity for if without else (false-edge PHI)

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_vm_llvm_if_without_else_phi() {
  local code='local v
v = "before"
if 0 {
  v = "then"
}
print(v)'
  check_parity -c "$code" "vm_llvm_if_without_else_phi"
}

run_test "vm_llvm_if_without_else_phi" test_vm_llvm_if_without_else_phi

