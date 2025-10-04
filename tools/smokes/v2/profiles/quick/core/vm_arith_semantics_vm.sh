#!/bin/bash
# vm_arith_semantics_vm.sh — Arithmetic semantics (Add/Sub/Mul/Div/Mod)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

if [ "${SMOKES_ENABLE_ROOTFIX:-0}" != "1" ]; then
  test_skip "vm_arith_semantics_vm (root-fix WIP)" "Enable with SMOKES_ENABLE_ROOTFIX=1" || exit 0
  exit 0
fi

TMP_DIR="/tmp/vm_arith_semantics_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
static box Main {
  main() {
    local a = -4
    local b = 6
    local c = 3
    local A = a + b     // 2
    local S = b - c     // 3
    local M = b * c     // 18
    local D = b / c     // 2
    local R = b % 4     // 2
    print("A="+(""+A))
    print("S="+(""+S))
    print("M="+(""+M))
    print("D="+(""+D))
    print("R="+(""+R))
    return 0
  }
}
NY

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tr -d '\r' | tail -n 5 | xargs echo)
expected="A=2 S=3 M=18 D=2 R=2"

test_name="vm_arith_semantics_vm"
compare_outputs "$expected" "$out" "$test_name" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

