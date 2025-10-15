#!/bin/bash
# vm_llvm_json_stringify_boundary_core.sh - VM↔LLVM parity: ArrayBox.toJSON simple boundary

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2

preflight_plugins || true

test_vm_llvm_json_stringify_boundary_core() {
  local code='
    static box Main { main() {
      local a = new ArrayBox();
      a.push("x"); a.push("y");
      print(a.toJSON());
      return 0
    }}
  '
  check_parity -c "$code" "vm_llvm_json_stringify_boundary_core"
}

run_test "vm_llvm_json_stringify_boundary_core" test_vm_llvm_json_stringify_boundary_core
