#!/bin/bash
# vm_bitops_semantics_vm.sh — Bitwise/Shifts semantics (And/Or/Xor/Shl/Shr)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

if [ "${SMOKES_ENABLE_ROOTFIX:-0}" != "1" ]; then
  test_skip "vm_bitops_semantics_vm (root-fix WIP)" "Enable with SMOKES_ENABLE_ROOTFIX=1" || exit 0
  exit 0
fi

TMP_DIR="/tmp/vm_bitops_semantics_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
static box Main {
  main() {
    local a = 6     // 110
    local b = 3     // 011
    local A = a & b // 2
    local O = a | b // 7
    local X = a ^ b // 5
    local L = 1 << 2 // 4
    local R = 8 >> 1 // 4
    print("A="+(""+A))
    print("O="+(""+O))
    print("X="+(""+X))
    print("L="+(""+L))
    print("R="+(""+R))
    return 0
  }
}
NY

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tr -d '' | tail -n 5 | xargs echo)
expected="A=2 O=7 X=5 L=4 R=4"

test_name="vm_bitops_semantics_vm"
compare_outputs "$expected" "$out" "$test_name" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
