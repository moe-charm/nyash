#!/bin/bash
# parity_q_compare_vm_llvm.sh — VM ↔ LLVM parity: comparisons + branch

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2
preflight_plugins || exit 2

read -r -d '' code <<'SRC'
if 3 > 2 {
  if 2 < 3 {
    print("OK")
  } else {
    print("NG")
  }
} else {
  print("NG")
}
SRC

check_parity -c "$code" "parity_q_compare_vm_llvm"
