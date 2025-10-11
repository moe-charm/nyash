#!/bin/bash
# parity_q_add_vm_llvm.sh — VM ↔ LLVM parity: arithmetic precedence

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2
preflight_plugins || exit 2

code='print(1 + 2 * 3)'
check_parity -c "$code" "parity_q_add_vm_llvm"
