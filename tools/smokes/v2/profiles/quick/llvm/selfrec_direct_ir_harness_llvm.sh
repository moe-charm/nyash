#!/bin/bash
# selfrec_direct_ir_harness_llvm.sh — Verify self-recursive direct call removes box ops in IR

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
if [[ "${SMOKES_ENABLE_LLVM_SELFREC:-}" != "1" ]]; then
  test_skip "selfrec_direct_ir_harness_llvm gated; set SMOKES_ENABLE_LLVM_SELFREC=1"; exit 0
fi

TMP_DIR="/tmp/selfrec_direct_ir_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"
IR_OUT="$TMP_DIR/ir.ll"

cat > "$SRC" << 'EOF'
static box Main {
  fib(n) {
    if n < 2 { return n }
    return me.fib(n - 1) + me.fib(n - 2)
  }
  main() {
    // small n to keep IR short
    return me.fib(4)
  }
}
EOF

# Run with LLVM harness and dump IR
# Run harness directly to ensure IR dump is produced
PYTHONPATH="${PYTHONPATH:-$NYASH_ROOT}" \
NYASH_LLVM_USE_HARNESS=1 \
NYASH_NY_LLVM_COMPILER="$NYASH_ROOT/target/release/ny-llvmc" \
NYASH_EMIT_EXE_NYRT="$NYASH_ROOT/target/release" \
NYASH_LLVM_DUMP_IR="$IR_OUT" \
NYASH_MIR_SELFREC_DIRECT=1 \
NYASH_CLI_VERBOSE=1 \
"$NYASH_BIN" --backend llvm "$SRC" >/dev/null 2>&1 || true

IR_FILE="$IR_OUT"
if [ ! -s "$IR_FILE" ]; then
  # Fallback to default harness dump location when verbose
  IR_FILE="$NYASH_ROOT/tmp/nyash_harness.ll"
fi
if [ ! -s "$IR_FILE" ]; then
  test_fail "IR dump not generated" "tried: $IR_OUT and $NYASH_ROOT/tmp/nyash_harness.ll"; rm -rf "$TMP_DIR"; exit 1
fi

# Assert that direct self call marker exists in IR (Phase 1)
if ! grep -q 'direct_self_global' "$IR_FILE"; then
  test_fail "direct_self_global marker not found in IR (self-rec direct not applied)"; rm -rf "$TMP_DIR"; exit 1
fi

test_pass "selfrec_direct_ir_harness_llvm"
rm -rf "$TMP_DIR"
exit 0
