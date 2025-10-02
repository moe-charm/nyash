#!/bin/bash
# timer_now_ms_harness.sh — TimerBox.now_ms parity between VM and LLVM harness

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

export NYASH_LLVM_USE_HARNESS=1
TEST_PATH="$NYASH_ROOT/apps/tests/core/timer_now_ms.hako"

run_timer_now_ms_harness() {
  ensure_hako_toml
  local vm_out
  vm_out=$(run_nyash_vm "$TEST_PATH" --dev | awk '/^Result:/{print $0}' | head -n 1 | tr -d '\r' | xargs)
  compare_outputs "Result: OK" "$vm_out" "timer_now_ms_vm_baseline" || return 1

  local llvm_out
  llvm_out=$(run_nyash_llvm "$TEST_PATH" --dev | awk '/^Result:/{print $0}' | head -n 1 | tr -d '\r' | xargs)
  if [ -z "$llvm_out" ]; then
    test_skip "timer_now_ms_harness" "LLVM harness unavailable" || true
    return 0
  fi
  compare_outputs "$vm_out" "$llvm_out" "timer_now_ms_vm_vs_llvm" || return 1
  return 0
}

run_test "timer_now_ms_harness" run_timer_now_ms_harness
