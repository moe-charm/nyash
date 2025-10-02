#!/bin/bash
# timer_now_ms_vm.sh — Verify TimerBox.now_ms via Rust VM

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TEST_PATH="$NYASH_ROOT/apps/tests/core/timer_now_ms.hako"

run_timer_now_ms_vm() {
  ensure_hako_toml
  local output
  output=$(run_nyash_vm "$TEST_PATH" --dev | awk '/^Result:/{print $0}' | head -n 1 | tr -d '\r' | xargs)
  compare_outputs "Result: OK" "$output" "timer_now_ms_vm"
}

run_test "timer_now_ms_vm" run_timer_now_ms_vm
