#!/bin/bash
# llvm_core_if_compare_vm_llvm.sh — Core: If-Compare parity between VM and LLVM harness

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

if [[ "${SMOKES_ENABLE_LLVM_IFCMP:-}" != "1" ]]; then
  test_skip "LLVM if-compare (set SMOKES_ENABLE_LLVM_IFCMP=1)"
  exit 0
fi

TMP_DIR="/tmp/llvm_core_if_compare_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/if_cmp.nyash"

cat > "$SRC" << 'NYEOF'
static box Main {
  main() {
    if (5 < 7) { return 1 } else { return 0 }
  }
}
NYEOF

out_vm=$(run_nyash_vm "$SRC")
out_llvm=$(run_nyash_llvm "$SRC")

v_vm=$(echo "$out_vm" | tail -n 1 | tr -d '
' | xargs)
v_llvm=$(echo "$out_llvm" | tail -n 1 | tr -d '
' | xargs)
compare_outputs "$v_vm" "$v_llvm" "llvm_core_if_compare_vm_llvm" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
