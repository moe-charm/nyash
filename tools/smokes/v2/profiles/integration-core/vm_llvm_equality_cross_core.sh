#!/bin/bash
# vm_llvm_equality_cross_core.sh — Parity: cross-type equality should be false

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_llvm_or_skip || exit 0
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/vm_llvm_equality_cross_core_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Main {
  main() {
    if ("1" == 1) { print("ng") } else { print("ok") }
    return 0
  }
}
EOF

out_vm=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | grep -v '^Result: ')
NYASH_LLVM_USE_HARNESS=1 out_llvm=$(run_nyash_llvm "$TMP_DIR/driver.nyash" --dev | grep -v '^Result: ')
compare_outputs "$out_vm" "$out_llvm" "vm_llvm_equality_cross_core" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

