#!/bin/bash
# phi_invariants_nested_diamond.sh — LLVM: nested diamond PHI invariants
source "$(dirname "$0")/../../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2
TMP_DIR="/tmp/phi_invariants_nested_diamond_$$"; mkdir -p "$TMP_DIR"; SRC="$TMP_DIR/nested.nyash"; IR_OUT="$TMP_DIR/ir.ll"
cat > "$SRC" << 'NYEOF'
static box Main {
  main() {
    local x = 0
    if (1 < 2) {
      if (2 < 3) { x = 10 } else { x = 11 }
    } else {
      if (3 < 4) { x = 12 } else { x = 13 }
    }
    return x
  }
}
NYEOF
export NYASH_LLVM_DUMP_IR="$IR_OUT"
out=$(run_nyash_llvm "$SRC")
if [ ! -s "$IR_OUT" ]; then test_skip "phi_invariants_nested_diamond (no IR dump)"; rm -rf "$TMP_DIR"; exit 0; fi
if ! grep -q " phi " "$IR_OUT"; then log_error "nested_diamond: no PHI"; rm -rf "$TMP_DIR"; exit 1; fi
if grep -q "phi[[:space:]]\+[a-z0-9_\*]\+[[:space:]]*\[\]" "$IR_OUT"; then log_error "nested_diamond: empty PHI"; rm -rf "$TMP_DIR"; exit 1; fi
log_success "nested_diamond: PHI present and non-empty"; rm -rf "$TMP_DIR"; exit 0
