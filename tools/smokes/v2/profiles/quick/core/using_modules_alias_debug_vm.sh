#!/bin/bash
# using_modules_alias_debug_vm.sh — [modules] resolver E2E: selfhost.compiler.debug

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
export NYASH_ENABLE_USING=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/using_modules_alias_debug_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'NYEOF'
using selfhost.compiler.debug as DebugBox

static box Main {
  main() {
    local d = new DebugBox()
    d.set_enabled(0)
    d.log("hello")  // should not print
    print("ok")
    return 0
  }
}
NYEOF

raw_output=$(run_nyash_vm "$SRC")
result=$(echo "$raw_output" | tr -d '\r' | tail -n 1 | xargs)
if [ "$result" = "ok" ]; then
  log_success "using_modules_alias_debug_vm resolved selfhost.compiler.debug"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "using_modules_alias_debug_vm expected ok, got: ${result:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi
