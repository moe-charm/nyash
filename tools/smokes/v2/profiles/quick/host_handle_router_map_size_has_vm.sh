#!/bin/bash
# host_handle_router_map_size_has_vm.sh — Force Map.size/has via HostHandle slots (200/202)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_host_handle_router_map_size_has_vm() {
  local code=$'static box Main {\n  main() {\n    local m = new MapBox()\n    m.set("a", 10)\n    local s = m.size()\n    local h1 = m.has("a")\n    local h2 = m.has("b")\n    print("" + s + "," + h1 + "," + h2)\n    return 0\n  }\n}\n'
  NYASH_MAP_SIZE_FORCE_HOST=1 NYASH_MAP_HAS_FORCE_HOST=1 out=$(run_nyash_vm -c "$code" | tail -n 1 | tr -d '\r')
  compare_outputs "1,true,false" "$out" "host_handle_router_map_size_has_vm" || return 1
  return 0
}

run_test "host_handle_router_map_size_has_vm" test_host_handle_router_map_size_has_vm

