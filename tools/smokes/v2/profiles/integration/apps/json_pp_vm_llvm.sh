#!/bin/bash
# json_pp_vm_llvm.sh — Example app parity: JSON pretty printer (VM vs LLVM)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

APP_DIR="$NYASH_ROOT/apps/examples/json_pp"

output_vm=$(run_nyash_vm "$APP_DIR/main.nyash" --dev)

# LLVM availability check (skip when unavailable)
if ! "$NYASH_BIN" --version 2>/dev/null | grep -q "features.*llvm"; then
  test_skip "LLVM backend not available in this build"; exit 0
fi

NYASH_LLVM_USE_HARNESS=1 output_llvm=$(run_nyash_llvm "$APP_DIR/main.nyash" --dev)

compare_outputs "$output_vm" "$output_llvm" "json_pp_vm_llvm" || exit 1

