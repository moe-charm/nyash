#!/usr/bin/env bash
# mircall_callable_argv_reconstruct_vm.sh — methodRef.call(argv)（arity>0）argv再構成（MirCall 正常系）

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mircall_callable_argv_reconstruct_vm() {
  if [ "${NYASH_PLUGIN_POLICY:-auto}" != "off" ]; then
    test_skip "requires NYASH_PLUGIN_POLICY=off"; return 0
  fi
  # Build Map, set a key, then call has/1 via methodRef.call(["a"]) where argv is locally constructed
  local code=$'static box Main {\n  main() {\n    local m = new MapBox()\n    m.set("a", 1)\n    local mr = MapBox.methodRef(m, "has", 1)\n    local argv = new ArrayBox()\n    argv.push("a")\n    if mr.call(argv) != true { return 301 }\n    return 0\n  }\n}\n'
  NYASH_PLUGIN_POLICY=off out=$(run_nyash_vm -c "$code" 2>&1 | tail -n 1 | tr -d '\r')
  if [ "$out" = "OK" ] || [ -z "$out" ]; then
    test_pass mircall_callable_argv_reconstruct_vm
  else
    echo "$out"; test_fail "unexpected output"; return 1
  fi
}

run_test mircall_callable_argv_reconstruct_vm test_mircall_callable_argv_reconstruct_vm

