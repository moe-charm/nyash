#!/bin/bash
# parity_m2_const_ret_vm_llvm.sh — Quick parity: const→ret prints 42 (VM vs LLVM)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2
preflight_plugins || exit 2

# Keep this minimal to guard parity in quick
code='print(42)'
check_parity -c "$code" "parity_m2_const_ret_vm_llvm"
