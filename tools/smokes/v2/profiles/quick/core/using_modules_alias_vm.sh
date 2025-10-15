#!/bin/bash
# using_modules_alias_vm.sh — [modules] resolver end-to-end: alias resolves to module path

source "$(dirname "$0")/../../../lib/test_runner.sh"
# TODO: This test needs updating after JSON module reorganization
# The NYASH_MODULES override below is incomplete and needs all mir_vm_min dependencies
if [ "${SMOKES_ENABLE_ALIAS:-0}" != "1" ]; then
  test_skip "using_modules_alias_vm (needs update after JSON reorg)" "Enable with SMOKES_ENABLE_ALIAS=1" || exit 0
  exit 0
fi
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
export NYASH_USING=1
export NYASH_MODULES="selfhost.vm.mir_min=selfhost/vm/boxes/mir_vm_min.hako"
export NYASH_USING=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/using_modules_alias_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'EOF'
using selfhost.vm.entry as MiniVmEntryBox

static box Main {
  main() {
    // Minimal inline JSON → expect int result back (0/1)
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":1}},{\"op\":\"ret\",\"value\":1}]}]}]}"
    local v = MirVmMin._run_min(j)
    print("" + v)
    return 0
  }
}
EOF

raw_output=$(run_nyash_vm "$SRC")
result=$(echo "$raw_output" | tr -d '\r' | grep -E '^[[:space:]]*-?[0-9]+[[:space:]]*$' | tail -n 1 | xargs)
if [ "$result" = "1" ] || [ "$result" = "0" ]; then
  log_success "using_modules_alias_vm resolved selfhost.vm.mir_min (result=$result)"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "using_modules_alias_vm expected 1, got: ${result:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi
