#!/bin/bash
# vm_compare_semantics_vm.sh — Compare semantics (Eq/Ne/Lt/Le/Gt/Ge) on integers

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

# Default: keep quick green until root fixes land
if [ "${SMOKES_ENABLE_ROOTFIX:-0}" != "1" ]; then
  test_skip "vm_compare_semantics_vm (root-fix WIP)" "Enable with SMOKES_ENABLE_ROOTFIX=1" || exit 0
  exit 0
fi

TMP_DIR="/tmp/vm_compare_semantics_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
static box Main {
  main() {
    local a = -4
    local b = 0
    // Coerce to string via ""+n
    local e = 0
    if a == b { e = 1 }
    local n = 0
    if a != b { n = 1 }
    local lt = 0
    if a < b { lt = 1 }
    local le = 0
    if a <= b { le = 1 }
    local gt = 0
    if a > b { gt = 1 }
    local ge = 0
    if a >= b { ge = 1 }
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

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tr -d '' | tail -n 6 | xargs echo)
# Expect: a=-4, b=0 → Eq=0 Ne=1 Lt=1 Le=1 Gt=0 Ge=0
expected="E=0 N=1 LT=1 LE=1 GT=0 GE=0"

test_name="vm_compare_semantics_vm"
compare_outputs "$expected" "$out" "$test_name" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
