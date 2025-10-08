#!/bin/bash
# vm_llvm_if_else_phi_core.sh - If/Else + PHI parity (no plugins)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2

preflight_plugins || true

test_vm_llvm_parity_if_else() {
  check_parity -c '
    static box Main { main() {
      local x = 10
      if x > 5 { print("big") } else { print("small") }
      if x > 100 { print("huge") } else { print("tiny") }
      return 0
    }}
  ' "vm_llvm_if_else_phi_core"
}

run_test "vm_llvm_if_else_phi_core" test_vm_llvm_parity_if_else

