#!/usr/bin/env bash
# mircall_map_delete_clear_vm.sh — Map.delete/clear 正常系（MirCall 経路）

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mircall_map_delete_clear_vm() {
  if [ "${NYASH_PLUGIN_POLICY:-auto}" != "off" ]; then
    test_skip "requires NYASH_PLUGIN_POLICY=off"; return 0
  fi
  local code=$'static box Main {\n  main() {\n    local m = new MapBox()\n    m.set("k", 1)\n    m.delete("k")\n    if m.has("k") != false { return 241 }\n    if m.size() != 0 { return 242 }\n    m.set("x", 1)\n    m.set("y", 2)\n    m.clear()\n    if m.size() != 0 { return 243 }\n    return 0\n  }\n}\n'
  NYASH_PLUGIN_POLICY=off out=$(run_nyash_vm -c "$code" 2>&1 | tail -n 1 | tr -d '\r')
  if [ "$out" = "OK" ] || [ -z "$out" ]; then
    test_pass mircall_map_delete_clear_vm
  else
    echo "$out"; test_fail "unexpected output"; return 1
  fi
}

run_test mircall_map_delete_clear_vm test_mircall_map_delete_clear_vm
