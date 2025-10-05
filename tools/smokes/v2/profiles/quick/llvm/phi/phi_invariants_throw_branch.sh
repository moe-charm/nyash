#!/bin/bash
# phi_invariants_throw_branch.sh — LLVM harness: if-then throw branch should not produce malformed PHI

source "$(dirname "$0")/../../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/phi_invariants_throw_branch_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/throw_if.nyash"
IR_OUT="$TMP_DIR/ir.ll"

cat > "$SRC" << 'NY'
static box Main {
  main() {
    local x = 0
    if (true) { throw 42 } else { x = 7 }
    return x
  }
}
NY

# Request IR dump if supported
export NYASH_LLVM_DUMP_IR="$IR_OUT"
raw_output=$(run_nyash_llvm "$SRC")

if [ ! -s "$IR_OUT" ]; then
  test_skip "phi_invariants_throw_branch (no IR dump available)" "Harness not dumping IR in this build"
  rm -rf "$TMP_DIR"
  exit 0
fi

# The goal here is minimal: ensure no empty PHI is present.
if grep -q "phi[[:space:]]\+[a-z0-9_\*]\+[[:space:]]*\[\]" "$IR_OUT"; then
  log_error "phi_invariants_throw_branch: empty PHI detected"
  rm -rf "$TMP_DIR"
  exit 1
fi

log_success "phi_invariants_throw_branch: no empty PHI in IR (throw path pruned or handled)"
rm -rf "$TMP_DIR"
exit 0
