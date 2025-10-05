#!/bin/bash
# using_modules_nested_alias_vm.sh — [modules] resolver E2E: nested alias resolution

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
export NYASH_USING_AST=1
# Provide base modules; we'll alias in two steps: selfhost.vm → VM, VM.mir_min → MirVmMin
# Provide a minimal working set for MirVmMin to operate
export NYASH_MODULES="selfhost.vm.mir_min=apps/selfhost/vm/boxes/mir_vm_min.hako,selfhost.vm.handlers=apps/selfhost/vm/boxes/op_handlers.hako,selfhost.vm.json_frag=apps/selfhost/vm/boxes/json_frag.hako,selfhost.vm.string_scan=apps/selfhost/vm/boxes/string_scan.hako"
require_env || exit 2
# Always-on: nested alias is supported in dev with AST merge
preflight_plugins || exit 2

TMP_DIR="/tmp/using_modules_nested_alias_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'EOF'
using selfhost.vm as VM
using VM.mir_min as MirVmMin

static box Main {
  main() {
    // Minimal inline JSON → expect 1 as int back
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":1}},{\"op\":\"ret\",\"value\":1}]}]}]}"
    local v = MirVmMin._run_min(j)
    print("" + v)
    return 0
  }
}
EOF

raw_output=$(run_nyash_vm "$SRC")
result=$(echo "$raw_output" | tr -d '\r' | grep -E '^[[:space:]]*-?[0-9]+[[:space:]]*$' | tail -n 1 | xargs)
if [ "$result" = "1" ]; then
  log_success "using_modules_nested_alias_vm resolved nested alias (VM → MirVmMin)"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "using_modules_nested_alias_vm expected 1, got: ${result:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi
