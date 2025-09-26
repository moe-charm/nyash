#!/bin/bash
# vm_llvm_concat_string_number.sh - VM vs LLVM parity for string + number concatenation

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_vm_llvm_concat_string_number() {
  local code='print("ab" + 3)'
  check_parity -c "$code" "vm_llvm_concat_string_number"
}

run_test "vm_llvm_concat_string_number" test_vm_llvm_concat_string_number

