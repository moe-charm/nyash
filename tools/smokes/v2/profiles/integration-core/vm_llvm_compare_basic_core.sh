#!/bin/bash
# vm_llvm_compare_basic_core.sh - Core Compare parity (no plugins)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2

preflight_plugins || true

test_vm_llvm_parity_compare() {
  check_parity -c '
    static box Main { main() {
      if 1 == 1 { print("eq") } else { print("ne") }
      if 2 != 3 { print("ne") } else { print("eq") }
      return 0
    }}
  ' "vm_llvm_compare_basic_core"
}

run_test "vm_llvm_compare_basic_core" test_vm_llvm_parity_compare

