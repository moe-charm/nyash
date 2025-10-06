#!/bin/bash
# timer_now_ms_vm.sh — Verify TimerBox.now_ms via Rust VM

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DISABLE_PLUGINS=1

run_timer_now_ms_vm() {
  # Static TimerBox.now_ms; no plugins/using required
  local out
  out=$(run_nyash_vm -c 'static box Main { main() { if TimerBox.now_ms() >= 0 { print("ok") } else { print("ng") } return 0 } }' --dev | grep -v '^Result: ' | tail -n 1 | tr -d '\r' | xargs)
  compare_outputs "ok" "$out" "timer_now_ms_vm"
}

run_test "timer_now_ms_vm" run_timer_now_ms_vm
