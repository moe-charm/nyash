#!/usr/bin/env bash
# mircall_array_slice_join_vm.sh — Array.slice/join 正常系（MirCall 経路）

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mircall_array_slice_join_vm() {
  if [ "${NYASH_PLUGIN_POLICY:-auto}" != "off" ]; then
    test_skip "requires NYASH_PLUGIN_POLICY=off"; return 0
  fi
  local code=$'static box Main {\n  main() {\n    local a = new ArrayBox()\n    a.push("a")\n    a.push("b")\n    a.push("c")\n    local b = a.slice(1, 3)\n    if b.join(",") != "b,c" { return 231 }\n    if a.join("-") != "a-b-c" { return 232 }\n    return 0\n  }\n}\n'
  NYASH_PLUGIN_POLICY=off out=$(run_nyash_vm -c "$code" 2>&1 | tail -n 1 | tr -d '\r')
  if [ "$out" = "OK" ] || [ -z "$out" ]; then
    test_pass mircall_array_slice_join_vm
  else
    echo "$out"; test_fail "unexpected output"; return 1
  fi
}

run_test mircall_array_slice_join_vm test_mircall_array_slice_join_vm
