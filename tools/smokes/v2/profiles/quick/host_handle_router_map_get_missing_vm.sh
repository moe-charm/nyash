#!/bin/bash
# host_handle_router_map_get_missing_vm.sh — Force Map.get via HostHandle slot (203); missing -> null

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_host_handle_router_map_get_missing_vm() {
  local code=$'static box Main {\n  main() {\n    local m = new MapBox()\n    // no set; directly get missing\n    local v = m.get("missing")\n    print("" + v)\n    return 0\n  }\n}\n'
  NYASH_MAP_GET_FORCE_HOST=1 out=$(run_nyash_vm -c "$code" | tail -n 1 | tr -d '\r')
  compare_outputs "null" "$out" "host_handle_router_map_get_missing_vm" || return 1
  return 0
}

run_test "host_handle_router_map_get_missing_vm" test_host_handle_router_map_get_missing_vm

