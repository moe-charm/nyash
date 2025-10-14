#!/bin/bash
# strict_plugin_map_size_vm.sh — Strict plugin policy: Map.size via plugin (PASS)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_strict_plugin_map_size_vm() {
  local code=$'static box Main {\n  main() {\n    local m = new MapBox()\n    m.set("a", 1)\n    print(m.size())\n    return 0\n  }\n}'
  local out
  # Strict plugin policy; plugins must handle Map (no builtin fallback)
  out=$(HAKO_PLUGIN_POLICY=force run_nyash_vm -c "$code" | filter_noise)
  if echo "$out" | grep -qx '1'; then
    test_pass strict_plugin_map_size_vm
  else
    compare_outputs "1" "$out" "strict_plugin_map_size_vm"
  fi
}

run_test "strict_plugin_map_size_vm" test_strict_plugin_map_size_vm

