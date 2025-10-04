#!/bin/bash
# vm_compare_semantics_null_void_vm.sh — Compare semantics with void (uninitialized) under tolerance

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

if [ "${SMOKES_ENABLE_ROOTFIX:-0}" != "1" ]; then
  test_skip "vm_compare_semantics_null_void_vm (root-fix WIP)" "Enable with SMOKES_ENABLE_ROOTFIX=1" || exit 0
  exit 0
fi

TMP_DIR="/tmp/vm_compare_semantics_null_void_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
static box Main {
  main() {
    // Uninitialized local → Void (test-runner defaults NYASH_VM_TOLERATE_VOID=1)
    local v
    // Numeric tolerance: Void behaves as 0
    local e0 = 0
    if v == 0 { e0 = 1 }
    local n0 = 0
    if v != 0 { n0 = 1 }
    // String tolerance: Void behaves as empty string ""
    local es = 0
    if v == "" { es = 1 }
    local ns = 0
    if v != "" { ns = 1 }
    print("E0="+(""+e0))   // expect 1
    print("N0="+(""+n0))   // expect 0
    print("ES="+(""+es))   // expect 1
    print("NS="+(""+ns))   // expect 0
    return 0
  }
}
NY

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tr -d '\r' | tail -n 4 | xargs echo)
expected="E0=1 N0=0 ES=1 NS=0"

test_name="vm_compare_semantics_null_void_vm"
compare_outputs "$expected" "$out" "$test_name" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

