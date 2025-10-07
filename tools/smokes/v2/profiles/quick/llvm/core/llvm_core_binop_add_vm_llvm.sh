#!/bin/bash
# llvm_core_binop_add_vm_llvm.sh — Core: binop Add parity between VM and LLVM harness

source "$(dirname "$0")/../../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
export NYASH_USING=0
export NYASH_SKIP_TOML_ENV=1
export SMOKES_CLEAN_ENV=1

require_env || exit 2
preflight_plugins || exit 2

if [[ "${SMOKES_ENABLE_LLVM_CORE:-}" != "1" ]]; then
  test_skip "LLVM core parity (dev-only; set SMOKES_ENABLE_LLVM_CORE=1)"
  exit 0
fi
if [[ "${SMOKES_ENABLE_LLVM_ADD:-}" != "1" ]]; then
  test_skip "LLVM add parity (set SMOKES_ENABLE_LLVM_ADD=1)"
  exit 0
fi

TMP_DIR="/tmp/llvm_core_binop_add_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/binop_add.nyash"

cat > "$SRC" << 'NYEOF'
static box Main {
  main() {
    local a = 5
    local b = 7
    local c = a + b
    print(c)
    return c
  }
}
NYEOF

out_vm=$(run_nyash_vm "$SRC")
out_llvm=$(run_nyash_llvm "$SRC")

# Compare only the last numeric token for robustness
v_vm=$(echo "$out_vm" | grep -Eo '[0-9]+' | tail -n 1 | tr -d '\r' | xargs)
v_llvm=$(echo "$out_llvm" | grep -Eo '[0-9]+' | tail -n 1 | tr -d '\r' | xargs)
compare_outputs "$v_vm" "$v_llvm" "llvm_core_binop_add_vm_llvm" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
