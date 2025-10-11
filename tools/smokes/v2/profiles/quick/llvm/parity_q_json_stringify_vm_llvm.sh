#!/bin/bash
# parity_q_json_stringify_vm_llvm.sh — VM ↔ LLVM parity: Array.toJSON()

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2
preflight_plugins || exit 2

read -r -d '' code <<'SRC'
static box Main {
  main(){
    local a = new ArrayBox();
    a.push(1);
    a.push(2);
    print(a.toJSON());
    return 0;
  }
}
SRC

check_parity -c "$code" "parity_q_json_stringify_vm_llvm"
