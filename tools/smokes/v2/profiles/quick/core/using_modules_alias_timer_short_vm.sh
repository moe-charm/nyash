#!/bin/bash
# using_modules_alias_timer_short_vm.sh — [modules.aliases] E2E: timer → core.timer.TimerBox

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export NYASH_ALLOW_USING_FILE=1
export NYASH_USING_AST=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/using_modules_alias_timer_short_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'SRC_EOF'
using timer as TimerBox

static box Main {
  main() {
    // Just ensure it does not go backwards and returns number-like
    local a = TimerBox.now_ms()
    local b = TimerBox.now_ms()
    if (b < a) {
      print("ng")
      return 0
    }
    print("ok")
    return 0
  }
}
SRC_EOF

out_full=$(run_nyash_vm "$SRC")
if echo "$out_full" | grep -qi 'AST prelude merge is disabled\|using: file paths are disallowed'; then
  log_warn "SKIP using_modules_alias_timer_short_vm (using resolver disabled)"
  rm -rf "$TMP_DIR"; exit 0
fi
out=$(echo "$out_full" | grep -v '^Result: ' | tail -n 1 | tr -d '\r' | xargs)
if [ "$out" = "ok" ]; then
  log_success "using_modules_alias_timer_short_vm resolved timer alias"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "using_modules_alias_timer_short_vm expected ok, got: ${out:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi
