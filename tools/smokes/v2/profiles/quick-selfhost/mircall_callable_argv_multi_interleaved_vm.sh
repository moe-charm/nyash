#!/usr/bin/env bash
# mircall_callable_argv_multi_interleaved_vm.sh — methodRef.call with two-arg argv and interleaved ops

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mircall_callable_argv_multi_interleaved_vm() {
  if [ "${NYASH_PLUGIN_POLICY:-auto}" != "off" ]; then
    test_skip "requires NYASH_PLUGIN_POLICY=off"; return 0
  fi
  local code=$'static box Main {\n  main() {\n    local a = new ArrayBox()\n    a.push("A")\n    a.push("B")\n    a.push("C")\n    local mr = ArrayBox.methodRef(a, "slice", 2)\n    local argv = new ArrayBox()\n    argv.push(0)\n    // unrelated operations (ignored by reconstruction)\n    local dummy = "xx"\n    if dummy.size() < 0 { return 398 }\n    argv.push(2)\n    local b = mr.call(argv)\n    if b.join(",") != "A,B" { return 351 }\n    return 0\n  }\n}\n'
  NYASH_PLUGIN_POLICY=off out=$(run_nyash_vm -c "$code" 2>&1 | tail -n 1 | tr -d '\r')
  if [ "$out" = "OK" ] || [ -z "$out" ]; then
    test_pass mircall_callable_argv_multi_interleaved_vm
  else
    echo "$out"; test_fail "unexpected output"; return 1
  fi
}

run_test mircall_callable_argv_multi_interleaved_vm test_mircall_callable_argv_multi_interleaved_vm

