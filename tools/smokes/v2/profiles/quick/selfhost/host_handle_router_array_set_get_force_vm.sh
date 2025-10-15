#!/bin/bash
# host_handle_router_array_set_get_force_vm.sh — Force Array.set/get/len via HostHandle slots

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2

test_host_handle_router_array_set_get_force_vm() {
  local code=$'static box Main {\n  main() {\n    local a = new ArrayBox()\n    a.set(0, 1)\n    if a.size() != 1 { return 101 }\n    local v = a.get(0)\n    print("" + v)\n    return 0\n  }\n}\n'
  NYASH_ARRAY_FORCE_HOST=1 out=$(run_nyash_vm -c "$code" | tail -n 1 | tr -d '\r')
  compare_outputs "1" "$out" "host_handle_router_array_set_get_force_vm" || return 1
  return 0
}

run_test "host_handle_router_array_set_get_force_vm" test_host_handle_router_array_set_get_force_vm

