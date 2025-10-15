#!/bin/bash
# vm_llvm_unary_not_core.sh - Unary not parity (no plugins)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2

preflight_plugins || true

test_vm_llvm_parity_not() {
  check_parity -c '
    static box Main { main() {
      if ! (0 == 1) { print("t") } else { print("f") }
      if ! (1 == 1) { print("t") } else { print("f") }
      return 0
    }}
  ' "vm_llvm_unary_not_core"
}

run_test "vm_llvm_unary_not_core" test_vm_llvm_parity_not

