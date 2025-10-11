#!/bin/bash
# parity_q_loop_vm_llvm.sh — VM ↔ LLVM parity: loop sum

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2
preflight_plugins || exit 2

read -r -d '' code <<'SRC'
local s = 0
local i = 0
loop (i < 3) { s = s + 1; i = i + 1 }
print(s)
SRC

check_parity -c "$code" "parity_q_loop_vm_llvm"
