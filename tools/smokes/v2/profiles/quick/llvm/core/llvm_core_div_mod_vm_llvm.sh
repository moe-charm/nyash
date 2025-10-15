#!/bin/bash
# llvm_core_div_mod_vm_llvm.sh — Core: div/mod parity aggregated as single value
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
# Additional gate: div/mod parity can vary; keep off unless explicitly enabled
if [[ "${SMOKES_ENABLE_LLVM_DIVMOD:-}" != "1" ]]; then test_skip "LLVM div/mod parity (set SMOKES_ENABLE_LLVM_DIVMOD=1)"; exit 0; fi
TMP_DIR="/tmp/llvm_core_div_mod_$$"; mkdir -p "$TMP_DIR"; SRC="$TMP_DIR/div_mod.nyash"
cat > "$SRC" << 'NYEOF'
static box Main {
  main() {
    // aggregate: val = (q*100 + r) to compare as single number
    local a=13 local b=5
    local q = a / b
    local r = a % b
    local val = q*100 + r
    print(val)
    return val
  }
}
NYEOF
out_vm=$(run_nyash_vm "$SRC")
out_llvm=$(run_nyash_llvm "$SRC")
v_vm=$(echo "$out_vm" | tail -n 1 | tr -d '
' | xargs)
v_llvm=$(echo "$out_llvm" | tail -n 1 | tr -d '
' | xargs)
compare_outputs "$v_vm" "$v_llvm" "llvm_core_div_mod_vm_llvm" || { rm -rf "$TMP_DIR"; exit 1; }
rm -rf "$TMP_DIR"; exit 0
