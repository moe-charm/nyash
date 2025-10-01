#!/bin/bash
# selfhost_mir_cfg_branch_vm_llvm.sh — apps/dev/mir_cfg_branch_smoke parity (VM vs LLVM harness)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

APP_FILE="$NYASH_ROOT/apps/dev/mir_cfg_branch_smoke.nyash"

output_vm=$(run_nyash_vm "$APP_FILE" --dev)

# Harness-first: rely on run_nyash_llvm() to decide availability (harness or features)

NYASH_LLVM_USE_HARNESS=1 output_llvm=$(run_nyash_llvm "$APP_FILE" --dev)

compare_outputs "$output_vm" "$output_llvm" "selfhost_mir_cfg_branch_vm_llvm" || exit 1
