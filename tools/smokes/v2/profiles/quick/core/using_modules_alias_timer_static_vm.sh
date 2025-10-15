#!/bin/bash
# using_modules_alias_timer_static_vm.sh — [modules] resolver E2E: selfhost.core.timer (static call)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
export NYASH_USING=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/using_modules_alias_timer_static_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'NYEOF'
using selfhost.core.timer as TimerBox

static box Main {
  main() {
    local v1 = TimerBox.now_ms()
    local v2 = TimerBox.now_ms()
    if (v2 < v1) {
      print("ng")
      return 0
    }
    local v3 = TimerBox.now_ms()
    if (v3 < v2) {
      print("ng")
      return 0
    }
    print("ok")
    return 0
  }
}
NYEOF

raw_output=$(run_nyash_vm "$SRC")
result=$(echo "$raw_output" | grep -v '^Result: ' | tr -d '\r' | tail -n 1 | xargs)
if [ "$result" = "ok" ]; then
  log_success "using_modules_alias_timer_static_vm resolved selfhost.core.timer"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "using_modules_alias_timer_static_vm expected ok, got: ${result:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi
