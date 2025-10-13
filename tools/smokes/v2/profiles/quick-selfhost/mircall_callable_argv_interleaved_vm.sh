#!/usr/bin/env bash
# mircall_callable_argv_interleaved_vm.sh — methodRef.call(argv) with interleaved ops
# Goal: Ensure argv reconstruction tolerates unrelated ops between NewBox and call site.

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mircall_callable_argv_interleaved_vm() {
  if [ "${NYASH_PLUGIN_POLICY:-auto}" != "off" ]; then
    test_skip "requires NYASH_PLUGIN_POLICY=off"; return 0
  fi
  local code=$'static box Main {\n  main() {\n    local m = new MapBox()\n    m.set("a", 1)\n    local mr = MapBox.methodRef(m, "has", 1)\n    // argv construction with interleaved unrelated ops\n    local argv = new ArrayBox()\n    argv.push("a")\n    // unrelated pure ops (should be ignored by reconstruction)\n    local tmp = "x"\n    if tmp.size() < 0 { return 399 }\n    // final check\n    if mr.call(argv) != true { return 321 }\n    return 0\n  }\n}\n'
  NYASH_PLUGIN_POLICY=off out=$(run_nyash_vm -c "$code" 2>&1 | tail -n 1 | tr -d '\r')
  if [ "$out" = "OK" ] || [ -z "$out" ]; then
    test_pass mircall_callable_argv_interleaved_vm
  else
    echo "$out"; test_fail "unexpected output"; return 1
  fi
}

run_test mircall_callable_argv_interleaved_vm test_mircall_callable_argv_interleaved_vm

