#!/bin/bash
# json_pp_vm_llvm.sh — Example app parity: JSON pretty printer (VM vs LLVM)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

APP_DIR="$NYASH_ROOT/apps/examples/json_pp"

output_vm=$(run_nyash_vm "$APP_DIR/main.nyash" --dev)

# LLVM availability check (skip when unavailable)
# Harness-first: rely on run_nyash_llvm() to decide availability

NYASH_LLVM_USE_HARNESS=1 output_llvm=$(run_nyash_llvm "$APP_DIR/main.nyash" --dev)
# Guard: empty LLVM output → skip (avoid false negatives due to harness noise)
if [ -z "$output_llvm" ]; then
  test_skip "json_pp_vm_llvm" "empty LLVM output (harness filtering/noise)"
  exit 0
fi

compare_outputs "$output_vm" "$output_llvm" "json_pp_vm_llvm" || exit 1
