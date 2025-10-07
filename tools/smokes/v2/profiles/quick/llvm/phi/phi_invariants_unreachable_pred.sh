#!/bin/bash
# phi_invariants_unreachable_pred.sh — LLVM: PHI not malformed when a pred returns
source "$(dirname "$0")/../../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2
TMP_DIR="/tmp/phi_invariants_unreachable_pred_$$"; mkdir -p "$TMP_DIR"; SRC="$TMP_DIR/unreach.nyash"; IR_OUT="$TMP_DIR/ir.ll"
cat > "$SRC" << 'NYEOF'
static box Main {
  main() {
    local x = 1
    if (2 < 3) { return 7 } else { x = 9 }
    return x
  }
}
NYEOF
export NYASH_LLVM_DUMP_IR="$IR_OUT"
out=$(run_nyash_llvm "$SRC")
if [ ! -s "$IR_OUT" ]; then test_skip "phi_invariants_unreachable_pred (no IR dump)"; rm -rf "$TMP_DIR"; exit 0; fi
# Only check that we didn't generate empty PHIs
if grep -q "phi[[:space:]]\+[a-z0-9_\*]\+[[:space:]]*\[\]" "$IR_OUT"; then log_error "unreachable_pred: empty PHI"; rm -rf "$TMP_DIR"; exit 1; fi
log_success "unreachable_pred: no empty PHI"; rm -rf "$TMP_DIR"; exit 0
