#!/bin/bash
# vm_llvm_json_stringify_core.sh - VM↔LLVM parity for minimal JSON.stringify (no plugins)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2

preflight_plugins || true

test_vm_llvm_parity_json_stringify_core() {
  check_parity -c '
    static box Main { main(){
      // primitives
      print(JSON.stringify(42))
      print(JSON.stringify("ok"))
      // simple array/map sugar not used to avoid plugin deps
      return 0
    }}
  ' "vm_llvm_json_stringify_core"
}

run_test "vm_llvm_json_stringify_core" test_vm_llvm_parity_json_stringify_core

