#!/usr/bin/env bash
# host_handle_router_string_len_vm.sh — String.size HostHandle slot(300) 正常系（quick 観測）

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_host_handle_router_string_len_vm() {
  local code=$'static box Main {\n  main() {\n    local s = "nyash"\n    if s.size() != 5 { return 271 }\n    return 0\n  }\n}\n'
  NYASH_STRING_SIZE_FORCE_HOST=1 out=$(run_nyash_vm -c "$code" 2>&1 | tail -n 1 | tr -d '\r')
  if [ "$out" = "OK" ] || [ -z "$out" ]; then
    test_pass host_handle_router_string_len_vm
  else
    echo "$out"; test_fail "unexpected output"; return 1
  fi
}

run_test host_handle_router_string_len_vm test_host_handle_router_string_len_vm
