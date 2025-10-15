#!/bin/bash
# vm_llvm_compare_float_cross_core.sh - Float and cross-type compare parity

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2

preflight_plugins || true

test_vm_llvm_compare_float_cross_core() {
  check_parity -c '
    static box Main { main() {
      if 1.5 == 1.5 { print("feq") } else { print("fne") }
      if 1.5 == 2.0 { print("fne") } else { print("feq") }
      if "1" == 1 { print("bad") } else { print("xne") }
      return 0
    }}
  ' "vm_llvm_compare_float_cross_core"
}

run_test "vm_llvm_compare_float_cross_core" test_vm_llvm_compare_float_cross_core

