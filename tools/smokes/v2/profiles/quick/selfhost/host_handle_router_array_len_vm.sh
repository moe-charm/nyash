#!/bin/bash
# host_handle_router_array_len_vm.sh — Force Array.size via HostHandle slot (102)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2

test_host_handle_router_array_len_vm() {
  local code=$'static box Main {\n  main() {\n    local a = new ArrayBox()\n    a.push(1)\n    a.push(2)\n    print("" + a.size())\n    return 0\n  }\n}\n'
  # Force router to use host-slot 102 path for ArrayBox.size
  NYASH_ARRAY_SIZE_FORCE_HOST=1 out=$(run_nyash_vm -c "$code" | tail -n 1 | tr -d '\r')
  if [ "$out" != "2" ]; then
    compare_outputs "2" "$out" "host_handle_router_array_len_vm"
    return 1
  fi
  return 0
}

run_test "host_handle_router_array_len_vm" test_host_handle_router_array_len_vm

