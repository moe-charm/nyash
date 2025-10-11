#!/bin/bash
# selfhost_ret_undefined_register_vm.sh — ret references undefined register; expect error

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

TEST_main() {
  local j='{"version":0,"modules":[{"name":"m","functions":[{"name":"main","blocks":[{"id":0,"instructions":[{"op":"ret","value":9}]}]}]}]}'
  local prog='
using "selfhost/vm/boxes/mir_vm_min.hako" as MirVmMin
static box Main { main(){ local out = MirVmMin._run_min(j) return 0 } }
'
  local jstr=$(printf '%s' "$j" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')
  prog=${prog/j/$jstr}
  local out
  out=$(run_nyash_vm -c "$prog" 2>&1 | filter_noise)
  echo "$out" | grep -q "\[ERROR\] Undefined register ret" || { echo "$out"; return 1; }
  return 0
}

run_test "selfhost_ret_undefined_register_vm" TEST_main
