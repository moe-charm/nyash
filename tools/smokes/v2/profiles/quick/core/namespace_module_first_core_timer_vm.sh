#!/bin/bash
# namespace_module_first_core_timer_vm.sh — Verify module-first resolution for core.timer TimerBox

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
export NYASH_USING=1
export NYASH_NS_POLICY=module-first
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/namespace_module_first_core_timer_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'EOF'
using core.timer as TimerBox

static box Main {
  main() {
    // Call static TimerBox.now_ms and print 1 on success
    local ms = TimerBox.now_ms()
    print("1")
    return 0
  }
}

EOF

out=$(run_nyash_vm "$SRC" | tail -n 1 | tr -d '' | xargs)
expected="1"
compare_outputs "$expected" "$out" "namespace_module_first_core_timer_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

