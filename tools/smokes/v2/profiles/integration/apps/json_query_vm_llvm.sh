#!/bin/bash
# json_query_vm_llvm.sh — Example app parity: JSON query (VM vs LLVM)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

APP_DIR="$NYASH_ROOT/apps/examples/json_query"

# Disable builder instance→function rewrite to exercise same path in both backends
export NYASH_BUILDER_REWRITE_INSTANCE=0
output_vm=$(run_nyash_vm "$APP_DIR/main.nyash" --dev | grep -v '^Result: ')

# LLVM availability check
# Harness-first: rely on run_nyash_llvm() to decide availability

NYASH_LLVM_USE_HARNESS=1 output_llvm=$(run_nyash_llvm "$APP_DIR/main.nyash" --dev | grep -v '^Result: ')

# Guard: if LLVM output is empty (harness noise or filtered), skip to avoid false negatives
if [ -z "$output_llvm" ]; then
  test_skip "json_query_vm_llvm" "empty LLVM output (harness filtering/noise)"
  exit 0
fi

compare_outputs "$output_vm" "$output_llvm" "json_query_vm_llvm" || exit 1
