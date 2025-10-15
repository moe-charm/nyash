#!/bin/bash
# vm_llvm_op_eq_primitives_core.sh - VM↔LLVM parity for primitive ==/!=

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2

preflight_plugins || true

test_vm_llvm_op_eq_primitives_core() {
  local code='
    static box Main { main() {
      if 7 == 7 { print("t1") } else { print("f1") }
      if 7 == 8 { print("f2") } else { print("t2") }
      if 9 != 9 { print("f3") } else { print("t3") }
      if 9 != 8 { print("t4") } else { print("f4") }
      return 0
    }}
  '
  check_parity -c "$code" "vm_llvm_op_eq_primitives_core"
}

run_test "vm_llvm_op_eq_primitives_core" test_vm_llvm_op_eq_primitives_core
