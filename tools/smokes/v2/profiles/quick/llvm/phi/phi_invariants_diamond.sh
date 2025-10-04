#!/bin/bash
# phi_invariants_diamond.sh — LLVM harness: diamond PHI invariants (best-effort)

source "$(dirname "$0")/../../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/phi_invariants_diamond_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/diamond.nyash"
IR_OUT="$TMP_DIR/ir.ll"

cat > "$SRC" << 'EOF'
static box Main {
  main() {
    local x = 0
    if (true) { x = 1 } else { x = 2 }
    return x
  }
}
EOF

# Request IR dump if supported
export NYASH_LLVM_DUMP_IR="$IR_OUT"
raw_output=$(run_nyash_llvm "$SRC")

if [ ! -s "$IR_OUT" ]; then
  test_skip "phi_invariants_diamond (no IR dump available)" "Harness not dumping IR in this build"
  rm -rf "$TMP_DIR"
  exit 0
fi

# Basic checks: presence of phi and no empty PHI
if ! grep -q " phi " "$IR_OUT"; then
  log_error "phi_invariants_diamond: no PHI found in IR"
  rm -rf "$TMP_DIR"
  exit 1
fi

if grep -q "phi[[:space:]]\+[a-z0-9_\*]\+[[:space:]]*\[\]" "$IR_OUT"; then
  log_error "phi_invariants_diamond: empty PHI detected"
  rm -rf "$TMP_DIR"
  exit 1
fi

log_success "phi_invariants_diamond: IR contains PHI with non-empty incomings"
rm -rf "$TMP_DIR"
exit 0

