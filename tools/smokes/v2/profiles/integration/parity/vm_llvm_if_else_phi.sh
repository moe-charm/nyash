#!/bin/bash
# vm_llvm_if_else_phi.sh - VM vs LLVM parity for else-if PHI wiring

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_vm_llvm_if_else_phi() {
  local code='local score, x
score = 75
if score >= 80 {
  x = "A"
} else if score >= 60 {
  x = "B"
} else {
  x = "C"
}
print(x)'
  check_parity -c "$code" "vm_llvm_if_else_phi"
}

run_test "vm_llvm_if_else_phi" test_vm_llvm_if_else_phi

