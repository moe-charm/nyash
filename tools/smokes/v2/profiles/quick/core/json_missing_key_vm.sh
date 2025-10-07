#!/bin/bash
# json_missing_key_vm.sh — Missing 'cmp' key should error in Mini‑VM compare handler

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

TEST_main() {
  # Minimal MIR v0 with compare missing 'cmp'
  local seg='{"op":"compare","lhs":1,"rhs":2,"dst":3}'
  local prog='
using "apps/selfhost/vm/boxes/op_handlers.hako" as OpHandlersBox
static box Main { main(){ local regs = new MapBox() OpHandlersBox.handle_compare(seg, regs) return 0 } }
'
  # Inject JSON string literal (escaped) into program
  local jstr=$(printf '%s' "$seg" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')
  prog=${prog/seg/$jstr}
  local out
  out=$(run_nyash_vm -c "$prog" 2>&1 | filter_noise)
  echo "$out" | grep -q "\[ERROR\] Missing key: cmp" || { echo "$out"; return 1; }
  return 0
}

run_test "json_missing_key_vm" TEST_main
