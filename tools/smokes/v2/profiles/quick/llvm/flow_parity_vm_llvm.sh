#!/bin/bash
# flow_parity_vm_llvm.sh — VM ↔ LLVM parity for Flow calls

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_flow_parity() {
  export NYASH_ENABLE_FLOW=1
  local code='
  flow Math {
    add(a, b) { return a + b }
  }
  flow Main {
    main() {
      local v
      v = Math.add(7, 8)
      print(v)
      return 0
    }
  }
  '
  check_parity -c "$code" "flow_parity_vm_llvm" 30
}

run_test "flow_parity_vm_llvm" test_flow_parity

