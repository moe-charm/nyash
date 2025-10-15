#!/bin/bash
# vm_unary_neg_type_error_vm.sh — Neg on non-number should raise VM error (no output)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

if [ "${SMOKES_ENABLE_ROOTFIX:-0}" != "1" ]; then
  test_skip "vm_unary_neg_type_error_vm (root-fix WIP)" "Enable with SMOKES_ENABLE_ROOTFIX=1" || exit 0
  exit 0
fi

TMP_DIR="/tmp/vm_unary_neg_type_error_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
static box Main {
  main() {
    // Neg on string should error
    local s = "x"
    local y = -s
    print("SHOULD_NOT_PRINT")
    return 0
  }
}
NY

NYASH_VM_TOLERATE_VOID=0 out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tr -d '
' | tail -n 1)
expected=""

test_name="vm_unary_neg_type_error_vm"
compare_outputs "$expected" "$out" "$test_name" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
