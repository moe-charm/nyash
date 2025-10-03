#!/bin/bash
# using_modules_alias2_vm.sh — [modules] resolver E2E: alternate alias resolves and callable

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/using_modules_alias2_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'EOF'
using selfhost.vm.handlers as OpHandlersBox

static box Main {
  main() {
    // Call into handlers via alias and verify result is set
    local seg = "{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":7}}"
    local regs = new MapBox()
    OpHandlersBox.handle_const(seg, regs)
    local v = regs.get("1")
    print("" + v)
    return 0
  }
}
EOF

raw_output=$(run_nyash_vm "$SRC")
result=$(echo "$raw_output" | awk '/^[[:space:]]*-?[0-9]+[[:space:]]*$/ { val=$0 } END { gsub(/\r/,"",val); gsub(/^[[:space:]]+|[[:space:]]+$/ , "", val); print val }')
if [ "$result" = "7" ]; then
  log_success "using_modules_alias2_vm resolved selfhost.vm.handlers and executed"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "using_modules_alias2_vm expected 7, got: ${result:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi
