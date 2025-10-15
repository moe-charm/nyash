#!/usr/bin/env bash
# mircall_map_delete_missing_vm.sh — Map.delete(欠損キー)の安定挙動（MirCall 経路）

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mircall_map_delete_missing_vm() {
  if [ "${NYASH_PLUGIN_POLICY:-auto}" != "off" ]; then
    test_skip "requires NYASH_PLUGIN_POLICY=off"; return 0
  fi
  local code=$'static box Main {\n  main() {\n    local m = new MapBox()\n    // delete on missing should be no-op and not panic\n    m.delete("nope")\n    if m.size() != 0 { return 261 }\n    // after inserting other keys, deleting missing keeps size unchanged\n    m.set("a", 1)\n    m.set("b", 2)\n    m.delete("zzz")\n    if m.size() != 2 { return 262 }\n    return 0\n  }\n}\n'
  NYASH_PLUGIN_POLICY=off out=$(run_nyash_vm -c "$code" 2>&1 | tail -n 1 | tr -d '\r')
  if [ "$out" = "OK" ] || [ -z "$out" ]; then
    test_pass mircall_map_delete_missing_vm
  else
    echo "$out"; test_fail "unexpected output"; return 1
  fi
}

run_test mircall_map_delete_missing_vm test_mircall_map_delete_missing_vm

