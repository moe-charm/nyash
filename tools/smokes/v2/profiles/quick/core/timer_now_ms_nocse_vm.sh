#!/bin/bash
# timer_now_ms_nocse_vm.sh — TimerBox.now_ms is not CSE'd (delta > 0)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_DISABLE_PLUGINS=1
export NYASH_STATIC_CALL_TRACE=1
export NYASH_VM_TRACE=1

# Dev gate: enable explicitly to run this diagnostic smoke
if [ "${NYASH_TIMER_NOCSE_ENABLE:-0}" != "1" ]; then
  test_skip "timer_now_ms_nocse_vm" "diagnostic smoke disabled (set NYASH_TIMER_NOCSE_ENABLE=1)" || true
  exit 0
fi

# Try to run a minimal probe; if using resolution fails, SKIP gracefully.
PROBE=$(run_nyash_vm -c 'static box Main { main() { if TimerBox.now_ms() >= 0 { print("ok") } else { print("ng") } return 0 } }' --dev 2>/dev/null | tail -n 1 | tr -d '\r' | xargs || true)
if echo "$PROBE" | grep -qiE 'unknown|fail|panic|error'; then
  test_skip "timer_now_ms_nocse_vm" "TimerBox unavailable; skipping" || true
  exit 0
fi

OUT=$(run_nyash_vm -c '
static box Main {
  main() {
    // try up to N times to observe a positive delta
    local start = TimerBox.now_ms()
    local tries = 0
    loop(tries < 1000000) {
      local cur = TimerBox.now_ms()
      if cur > start { print("ok") return 0 }
      tries = tries + 1
    }
    print("ng")
    return 0
  }
}
' --dev | tail -n 1 | tr -d '\r' | xargs)

if [ "$OUT" = "ok" ]; then
  log_success "timer_now_ms_nocse_vm"
  exit 0
fi

log_error "timer_now_ms_nocse_vm output mismatch: expected ok, got: ${OUT}"
exit 1
