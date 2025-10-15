#!/bin/bash
# llvm_core_const_ret_vm_llvm.sh — Core: const→ret parity between VM and LLVM harness

source "$(dirname "$0")/../../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
export NYASH_USING=0
export NYASH_SKIP_TOML_ENV=1
export SMOKES_CLEAN_ENV=1
if [[ "${SMOKES_ENABLE_LLVM_CORE:-}" != "1" ]]; then
  test_skip "LLVM core (dev-only)"
  exit 0
fi
if [[ "${SMOKES_ENABLE_LLVM_CONSTRET:-}" != "1" ]]; then
  test_skip "LLVM const-ret parity (set SMOKES_ENABLE_LLVM_CONSTRET=1)"
  exit 0
fi
# Optional: gate unstable behaviour
if [[ "${SMOKES_ENABLE_LLVM_CONSTRET:-}" != "1" ]]; then
  : # keep default; test continues
fi
require_env || exit 2
preflight_plugins || exit 2

if [[ "${SMOKES_ENABLE_LLVM_CORE:-}" != "1" ]]; then
  test_skip "LLVM core parity (dev-only; set SMOKES_ENABLE_LLVM_CORE=1)"
  exit 0
fi

TMP_DIR="/tmp/llvm_core_const_ret_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/const_ret.nyash"

cat > "$SRC" << 'NYEOF'
static box Main {
  main() { print(42)  return 42 }
}
NYEOF

out_vm=$(run_nyash_vm "$SRC")
out_llvm=$(run_nyash_llvm "$SRC")

v_vm=$(echo "$out_vm" | tail -n 1 | tr -d '
' | xargs)
v_llvm=$(echo "$out_llvm" | tail -n 1 | tr -d '
' | xargs)
compare_outputs "$v_vm" "$v_llvm" "llvm_core_const_ret_vm_llvm" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
