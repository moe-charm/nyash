#!/bin/bash
# host_handle_router_map_set_get_vm.sh — Force Map.set/get via HostHandle slots (200/202/203/204)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2

test_host_handle_router_map_set_get_vm() {
  local code=$'static box Main {\n  main() {\n    local m = new MapBox()\n    m.set("x", 1)\n    local v1 = m.get("x")\n    m.set("x", v1 + 1)\n    local v2 = m.get("x")\n    print("" + v2)\n    return 0\n  }\n}\n'
  NYASH_MAP_FORCE_HOST=1 out=$(run_nyash_vm -c "$code" | tail -n 1 | tr -d '\r')
  compare_outputs "2" "$out" "host_handle_router_map_set_get_vm" || return 1
  return 0
}

run_test "host_handle_router_map_set_get_vm" test_host_handle_router_map_set_get_vm

