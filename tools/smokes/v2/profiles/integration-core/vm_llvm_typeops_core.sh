#!/bin/bash
# vm_llvm_typeops_core.sh - Type ops parity (is/cast minimal)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2

preflight_plugins || true

test_vm_llvm_typeops_core() {
  check_parity -c '
    static box Main { main() {
      local x = 42
      local y = "hi"
      if x is Integer { print("isInt") } else { print("notInt") }
      if y is String { print("isStr") } else { print("notStr") }
      // Cross-type is checks
      if x is String { print("bad") } else { print("ok") }
      return 0
    }}
  ' "vm_llvm_typeops_core"
}

run_test "vm_llvm_typeops_core" test_vm_llvm_typeops_core

