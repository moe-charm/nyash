#!/bin/bash
# vm_compare_semantics_strings_vm.sh — String compare semantics (Eq/Ne/Lt/Le/Gt/Ge)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

if [ "${SMOKES_ENABLE_ROOTFIX:-0}" != "1" ]; then
  test_skip "vm_compare_semantics_strings_vm (root-fix WIP)" "Enable with SMOKES_ENABLE_ROOTFIX=1" || exit 0
  exit 0
fi

TMP_DIR="/tmp/vm_compare_semantics_strings_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
static box Main {
  main() {
    local a = "abc"
    local b = "abd"
    local c = "abc"
    local e = 0
    if a == c { e = 1 }
    local n = 0
    if a != b { n = 1 }
    local lt = 0
    if a < b { lt = 1 }
    local le = 0
    if a <= c { le = 1 }
    local gt = 0
    if b > a { gt = 1 }
    local ge = 0
    if b >= a { ge = 1 }
    print("E="+(""+e))
    print("N="+(""+n))
    print("LT="+(""+lt))
    print("LE="+(""+le))
    print("GT="+(""+gt))
    print("GE="+(""+ge))
    return 0
  }
}
NY

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tr -d '\r' | tail -n 6 | xargs echo)
expected="E=1 N=1 LT=1 LE=1 GT=1 GE=1"

test_name="vm_compare_semantics_strings_vm"
compare_outputs "$expected" "$out" "$test_name" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

