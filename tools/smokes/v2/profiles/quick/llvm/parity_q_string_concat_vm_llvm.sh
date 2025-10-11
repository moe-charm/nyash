#!/bin/bash
# parity_q_string_concat_vm_llvm.sh — VM ↔ LLVM parity: string concat

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2
preflight_plugins || exit 2

code='local s = "a"; s = s + "b"; print(s)'
check_parity -c "$code" "parity_q_string_concat_vm_llvm"
