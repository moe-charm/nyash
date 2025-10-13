#!/usr/bin/env bash
# mircall_callable_noargs_vm.sh — methodRef.call([])（arity==0）正常系（MirCall）

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mircall_callable_noargs_vm() {
  if [ "${NYASH_PLUGIN_POLICY:-auto}" != "off" ]; then
    test_skip "requires NYASH_PLUGIN_POLICY=off"; return 0
  fi
  local code=$'static box Main {\n  main() {\n    local a = new ArrayBox()\n    a.push("x")\n    local mr = ArrayBox.methodRef(a, "size", 0)\n    if mr.call([]) != 1 { return 311 }\n    return 0\n  }\n}\n'
  NYASH_PLUGIN_POLICY=off out=$(run_nyash_vm -c "$code" 2>&1 | tail -n 1 | tr -d '\r')
  if [ "$out" = "OK" ] || [ -z "$out" ]; then
    test_pass mircall_callable_noargs_vm
  else
    echo "$out"; test_fail "unexpected output"; return 1
  fi
}

run_test mircall_callable_noargs_vm test_mircall_callable_noargs_vm

