#!/bin/bash
# vm_llvm_compare_le_ge_boundary_core.sh - VM↔LLVM parity for <=, >= at equality boundary

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2

preflight_plugins || true

test_vm_llvm_compare_le_ge_boundary_core() {
  local code='
    static box Main { main() {
      if 5 <= 5 { print("t1") } else { print("f1") }
      if 5 >= 5 { print("t2") } else { print("f2") }
      if 5 <  5 { print("f3") } else { print("t3") }
      if 5 >  5 { print("f4") } else { print("t4") }
      return 0
    }}
  '
  check_parity -c "$code" "vm_llvm_compare_le_ge_boundary_core"
}

run_test "vm_llvm_compare_le_ge_boundary_core" test_vm_llvm_compare_le_ge_boundary_core

