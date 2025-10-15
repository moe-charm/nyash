#!/bin/bash
# vm_llvm_op_eq_box_reflexive_core.sh - VM↔LLVM parity: BoxRef reflexive equality (plugins OFF for stability)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2

# Force plugins OFF for reflexive pointer equality stability
export HAKO_PLUGIN_POLICY=off
export NYASH_PLUGIN_POLICY=off

preflight_plugins || true

test_vm_llvm_op_eq_box_reflexive_core() {
  local code='
    box Ref { equals(o){ return me == o } }
    static box Main { main() {
      local a = new Ref();
      local b = a;
      if a == a { print("t1") } else { print("f1") }
      if a == b { print("t2") } else { print("f2") }
      return 0
    }}
  '
  check_parity -c "$code" "vm_llvm_op_eq_box_reflexive_core"
}

run_test "vm_llvm_op_eq_box_reflexive_core" test_vm_llvm_op_eq_box_reflexive_core
