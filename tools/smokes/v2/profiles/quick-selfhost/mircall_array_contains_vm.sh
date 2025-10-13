#!/usr/bin/env bash
# mircall_array_contains_vm.sh — Array.contains 正常系（MirCall 経路, quick-selfhost）

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mircall_array_contains_vm_selfhost() {
  if [ "${NYASH_PLUGIN_POLICY:-auto}" != "off" ]; then
    test_skip "requires NYASH_PLUGIN_POLICY=off"; return 0
  fi
  local code=$'static box Main {\n  main() {\n    local a = new ArrayBox()\n    a.push("a")\n    a.push("b")\n    a.push("c")\n    if a.contains("b") != true { return 281 }\n    if a.contains("x") != false { return 282 }\n    return 0\n  }\n}\n'
  NYASH_PLUGIN_POLICY=off out=$(run_nyash_vm -c "$code" 2>&1 | tail -n 1 | tr -d '\r')
  if [ "$out" = "OK" ] || [ -z "$out" ]; then
    test_pass mircall_array_contains_vm_selfhost
  else
    echo "$out"; test_fail "unexpected output"; return 1
  fi
}

run_test mircall_array_contains_vm_selfhost test_mircall_array_contains_vm_selfhost

