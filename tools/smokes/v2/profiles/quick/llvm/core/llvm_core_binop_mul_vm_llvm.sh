#!/bin/bash
# llvm_core_binop_mul_vm_llvm.sh — Core: binop Mul parity (VM vs LLVM)
source "$(dirname "$0")/../../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
export NYASH_USING=0
export NYASH_SKIP_TOML_ENV=1
export SMOKES_CLEAN_ENV=1
require_env || exit 2
preflight_plugins || exit 2
if [[ "${SMOKES_ENABLE_LLVM_CORE:-}" != "1" ]]; then test_skip "LLVM core (dev-only)"; exit 0; fi
# Additional gate: mul parity is experimental; keep off unless explicitly enabled
if [[ "${SMOKES_ENABLE_LLVM_MUL:-}" != "1" ]]; then test_skip "LLVM mul parity (set SMOKES_ENABLE_LLVM_MUL=1)"; exit 0; fi
TMP_DIR="/tmp/llvm_core_binop_mul_$$"; mkdir -p "$TMP_DIR"; SRC="$TMP_DIR/binop_mul.nyash"
cat > "$SRC" << 'NYEOF'
static box Main {
  main() {
    local c = 6 * 7
    print(c)
    return c
  }
}
NYEOF
out_vm=$(run_nyash_vm "$SRC")
out_llvm=$(run_nyash_llvm "$SRC")
v_vm=$(echo "$out_vm" | grep -Eo '[0-9]+' | tail -n 1 | xargs)
v_llvm=$(echo "$out_llvm" | grep -Eo '[0-9]+' | tail -n 1 | xargs)
compare_outputs "$v_vm" "$v_llvm" "llvm_core_binop_mul_vm_llvm" || { rm -rf "$TMP_DIR"; exit 1; }
rm -rf "$TMP_DIR"; exit 0
