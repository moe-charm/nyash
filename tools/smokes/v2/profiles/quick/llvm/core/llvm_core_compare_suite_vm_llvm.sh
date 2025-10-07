#!/bin/bash
# llvm_core_compare_suite_vm_llvm.sh — Core: compare ops suite aggregated
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
if [[ "${SMOKES_ENABLE_LLVM_COMPARE:-}" != "1" ]]; then test_skip "LLVM compare suite (set SMOKES_ENABLE_LLVM_COMPARE=1)"; exit 0; fi
TMP_DIR="/tmp/llvm_core_compare_suite_$$"; mkdir -p "$TMP_DIR"; SRC="$TMP_DIR/compare_suite.nyash"
cat > "$SRC" << 'NYEOF'
static box Main {
  main() {
    local x=5 local y=7
    local s = 0
    if (x == y) { s = s + 1 } else { s = s + 0 }
    if (x != y) { s = s + 1 } else { s = s + 0 }
    if (x <  y) { s = s + 1 } else { s = s + 0 }
    if (x <= y) { s = s + 1 } else { s = s + 0 }
    if (x >  y) { s = s + 1 } else { s = s + 0 }
    if (x >= y) { s = s + 1 } else { s = s + 0 }
    print(s)
    return s
  }
}
NYEOF
out_vm=$(run_nyash_vm "$SRC")
out_llvm=$(run_nyash_llvm "$SRC")
v_vm=$(echo "$out_vm" | tail -n 1 | tr -d '
' | xargs)
v_llvm=$(echo "$out_llvm" | tail -n 1 | tr -d '
' | xargs)
compare_outputs "$v_vm" "$v_llvm" "llvm_core_compare_suite_vm_llvm" || { rm -rf "$TMP_DIR"; exit 1; }
rm -rf "$TMP_DIR"; exit 0
