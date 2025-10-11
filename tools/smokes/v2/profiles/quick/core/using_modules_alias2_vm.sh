#!/bin/bash
# using_modules_alias2_vm.sh — [modules] resolver E2E: alternate alias resolves and callable

source "$(dirname "$0")/../../../lib/test_runner.sh"
# Gate: module-alias call into OpHandlersBox; enable explicitly to avoid profile differences.
if [ "${SMOKES_ENABLE_ALIAS2:-0}" != "1" ]; then
  log_warn "SKIP using_modules_alias2_vm (set SMOKES_ENABLE_ALIAS2=1 to run)"
  exit 0
fi
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export NYASH_MODULES="selfhost.vm.mir_min=selfhost/vm/boxes/mir_vm_min.hako,selfhost.vm.handlers=selfhost/vm/boxes/op_handlers.hako,selfhost.json.utils.json_frag=apps/selfhost/common/json/utils/json_frag.hako,selfhost.json.core.string_scan=apps/selfhost/common/json/core/string_scan.hako"
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
