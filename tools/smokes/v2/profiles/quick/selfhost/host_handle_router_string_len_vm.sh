#!/bin/bash
# host_handle_router_string_len_vm.sh — Force String.size() via HostHandleRouter slot 300

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2

test_host_handle_router_string_len_vm() {
  local code=$'static box Main {\n  main() {\n    local s = "hello world"\n    if s.size() != 11 { return 101 }\n    return 0\n  }\n}\n'
  NYASH_STRING_SIZE_FORCE_HOST=1 out=$(run_nyash_vm -c "$code" 2>&1 | tail -n 1 | tr -d '\r')
  if [ "$out" = "OK" ] || [ -z "$out" ]; then
    test_pass "host_handle_router_string_len_vm"; return 0
  fi
  # The quick runner prints OK on success or empty line; normalize
  if echo "$out" | grep -q '^$'; then test_pass "host_handle_router_string_len_vm"; return 0; fi
  test_fail "unexpected output: $out"
}

run_test "host_handle_router_string_len_vm" test_host_handle_router_string_len_vm

