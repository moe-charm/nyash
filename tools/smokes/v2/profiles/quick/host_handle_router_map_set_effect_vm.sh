#!/bin/bash
# host_handle_router_map_set_effect_vm.sh — Force Map.set via HostHandle slot (204); verify functional effect

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_host_handle_router_map_set_effect_vm() {
  local code=$'static box Main {\n  main() {\n    local m = new MapBox()\n    m.set("a", 1)\n    m.set("b", 2)\n    print("" + m.size())\n    return 0\n  }\n}\n'
  NYASH_MAP_SET_FORCE_HOST=1 NYASH_MAP_SIZE_FORCE_HOST=1 out=$(run_nyash_vm -c "$code" | tail -n 1 | tr -d '\r')
  compare_outputs "2" "$out" "host_handle_router_map_set_effect_vm" || return 1
  return 0
}

run_test "host_handle_router_map_set_effect_vm" test_host_handle_router_map_set_effect_vm
