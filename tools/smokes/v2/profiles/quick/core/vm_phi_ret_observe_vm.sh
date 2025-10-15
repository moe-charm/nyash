#!/bin/bash
# vm_phi_ret_observe_vm.sh — Observe PHI merge and Return on a simple branch.

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/vm_phi_ret_observe_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
static box Main {
  main() {
    local x = 1
    local y = 2
    local z
    if x < y { z = 10 } else { z = 20 }
    print("Z="+(""+z))
    return z
  }
}
NY

# Enable observation logs; filter output to only the program print
NYASH_VM_PHI_TRACE=1 NYASH_VM_RET_TRACE=1 out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | grep '^Z=' | tr -d '\r' | tail -n 1)
expected="Z=10"

test_name="vm_phi_ret_observe_vm"
compare_outputs "$expected" "$out" "$test_name" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

