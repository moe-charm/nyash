#!/bin/bash
# vm_llvm_hello_core.sh - Core VM vs LLVM parity (no plugins)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2

preflight_plugins || true  # plugins not required for core tests

test_vm_llvm_parity_hello() {
  check_parity -c 'print("Hello core!")' "vm_llvm_hello_core"
}

run_test "vm_llvm_hello_core" test_vm_llvm_parity_hello

