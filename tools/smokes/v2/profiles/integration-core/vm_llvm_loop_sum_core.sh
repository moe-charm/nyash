#!/bin/bash
# vm_llvm_loop_sum_core.sh - while loop sum parity (no plugins)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2

preflight_plugins || true

test_vm_llvm_parity_loop_sum() {
  check_parity -c '
    static box Main { main() {
      local i = 0
      local s = 0
      while (i < 5) { s = s + i; i = i + 1 }
      print(s)
      return 0
    }}
  ' "vm_llvm_loop_sum_core"
}

run_test "vm_llvm_loop_sum_core" test_vm_llvm_parity_loop_sum

