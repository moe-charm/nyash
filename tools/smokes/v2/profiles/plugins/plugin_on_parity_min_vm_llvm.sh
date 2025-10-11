#!/bin/bash
# plugin_on_parity_min_vm_llvm.sh — plugins profile: minimal LLVM run cross-check

source "$(dirname "$0")/../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
require_env || exit 2
preflight_plugins || exit 2

run_test_plugin_on_parity_min_vm_llvm() {
  local code=$'print("OK")
'
  local out
  out=$(run_nyash_llvm -c "$code")
  if ! echo "$out" | grep -qx 'OK'; then
    echo "FAIL: $out" >&2
    return 1
  fi
  return 0
}

run_test "plugin_on_parity_min_vm_llvm" run_test_plugin_on_parity_min_vm_llvm
