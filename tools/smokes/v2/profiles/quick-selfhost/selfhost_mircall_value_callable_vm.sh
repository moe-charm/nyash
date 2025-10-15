#!/bin/bash
# selfhost_mircall_value_callable_vm.sh — MirCall callee=Value calling a CallableBox

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

# Guard: Callable via Value callee is experimental in quick-selfhost; opt-in only
if [ "${SMOKES_ALLOW_CALLABLE_VALUE:-0}" != "1" ]; then
  SMOKES_SKIP_CUR_TEST=1; SMOKES_SKIP_REASON="Callable(Value) disabled in quick-selfhost";
fi

test_selfhost_mircall_value_callable_vm() {
  local code='
using "selfhost/hakorune-vm/hakorune_vm_core.hako" as HakoruneVmCore
using "selfhost/shared/common/string_helpers.hako" as StringHelpers

static box Main { main() {
  local mir = r#"{"functions":[{"name":"test","blocks":[{"id":0,"instructions":[{"op":"newbox","dst":1,"box_type":"ArrayBox"},{"op":"const","dst":2,"value":{"Integer":1}},{"op":"boxcall","dst":null,"box":1,"method":"push","args":[2]},{"op":"const","dst":3,"value":{"String":"size"}},{"op":"const","dst":4,"value":{"Integer":0}},{"op":"boxcall","dst":5,"box":1,"method":"methodRef","args":[3,4]},{"op":"mir_call","dst":6,"mir_call":{"callee":{"type":"Value","value":5},"args":[]}}, {"op":"ret","value":6}],"terminator":{"op":"ret","value":6}}]}]}"#
  local out = HakoruneVmCore.run(mir)
  print("" + out)
  return 0
}}
'
  out=$(run_nyash_vm -c "$code")
  if [ "$out" = "1" ]; then
    test_pass "selfhost_mircall_value_callable_vm"
  else
    echo "$out"; test_fail "expected 1 from array.size() via CallableBox"; return 1
  fi
}

run_test "selfhost_mircall_value_callable_vm" test_selfhost_mircall_value_callable_vm
