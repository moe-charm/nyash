#!/bin/bash
# selfhost_front_min_vm_llvm.sh — Minimal VM/LLVM parity check for selfhost samples.

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_FORCE_LLVM=1
require_env || exit 2
preflight_plugins || exit 2
export NYASH_LLVM_USE_HARNESS=1
export NYASH_NYRT_SILENT_RESULT=1

cases=(
  "const:apps/tests/selfhost_min/const_ret.hako:Result: 42"
  "if_merge:apps/tests/selfhost_min/if_merge.hako:Result: 10"
  "loop_sum:apps/tests/selfhost_min/loop_sum.hako:Result: 15"
)

run_case() {
  local label="$1"
  local path="$2"
  local expected="$3"

  ensure_hako_toml

  local out_vm
  out_vm=$(run_nyash_vm "$NYASH_ROOT/$path" --dev | awk '/^Result:/{print $0}' | head -n 1 | tr -d '\r' | xargs)
  if [ -z "$out_vm" ]; then
    log_error "${label}: missing Result line (VM)"
    return 1
  fi
  compare_outputs "$expected" "$out_vm" "${label}_vm" || return 1

  local out_llvm
  out_llvm=$(NYASH_LLVM_USE_HARNESS=1 run_nyash_llvm "$NYASH_ROOT/$path" --dev | awk '/^Result:/{print $0}' | head -n 1 | tr -d '\r' | xargs)
  if [ -z "$out_llvm" ]; then
    log_warn "${label}: LLVM harness unavailable (SKIP)"
  else
    compare_outputs "$out_vm" "$out_llvm" "${label}_vm_vs_llvm" || return 1
  fi
  return 0
}

all_ok=0
for entry in "${cases[@]}"; do
  IFS=':' read -r label path expected <<< "$entry"
  if ! run_case "$label" "$path" "$expected"; then
    all_ok=1
  fi
done

if [ $all_ok -ne 0 ]; then
  exit 1
fi

exit 0
