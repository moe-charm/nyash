#!/bin/bash
# plugin_parity_min_vm_llvm.sh — plugins profile: minimal VM↔LLVM parity (plugins off for parity)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2
preflight_plugins || exit 2

read -r -d '' code <<'SRC'
print("OK")
SRC

check_parity -c "$code" "plugin_parity_min_vm_llvm"
