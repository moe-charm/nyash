#!/bin/bash
# json_missing_key_vm.sh — Missing 'cmp' key should error in Mini‑VM compare handler

source "$(dirname "$0")/../../../lib/test_runner.sh"
if [ "${SMOKES_ENABLE_OP_HANDLERS:-0}" != "1" ]; then
  echo "SKIP: enable with SMOKES_ENABLE_OP_HANDLERS=1" >&2
  exit 0
fi
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export NYASH_USING_AST=1
require_env || exit 2
preflight_plugins || exit 2

TEST_main() {
  # Minimal MIR v0 with compare missing 'cmp'
  local seg='{"op":"compare","lhs":1,"rhs":2,"dst":3}'
  local prog='
using "selfhost/vm/boxes/op_handlers.hako" as OpHandlersBox
static box Main { main(){ local regs = new MapBox() OpHandlersBox.handle_compare(seg, regs) return 0 } }
'
  # Inject JSON string literal (escaped) into program
  local jstr=$(printf '%s' "$seg" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')
  prog=${prog/seg/$jstr}
  local out
  out=$(run_nyash_vm -c "$prog" 2>&1 | filter_noise)
  if echo "$out" | grep -q 'using: file paths are disallowed'; then
    log_warn "SKIP json_missing_key_vm (file path using disallowed in this env)"
    return 0
  fi
  # Accept either strict tagged error or a relaxed message containing 'Missing key'
  echo "$out" | grep -q -E '(\[ERROR\][[:space:]]+)?Missing key' || { echo "$out"; return 1; }
  return 0
}

run_test "json_missing_key_vm" TEST_main
