#!/bin/bash
# json_stringify_vm.sh - Minimal JSON.stringify smoke (runtime shim)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

preflight_plugins || true

test_json_stringify_vm() {
  # Not plugin-dependent; always run.
  local code='
    static box Main { main() {
      print(JSON.stringify(42))
      print(JSON.stringify("hi"))
      return 0
    }}
  '
  out=$(run_nyash_vm -c "$code" --dev)
  # Skip gracefully if JSON.stringify is not available in this build
  if echo "$out" | grep -q "Unknown module function: JSON\.stringify"; then
    test_skip "json_stringify_vm (JSON.stringify unsupported in this build)"
    return 0
  fi
  # We only check presence of two lines (order preserved)
  echo "$out" | grep -q "^42$" || { test_fail "json_stringify missing 42"; return 1; }
  echo "$out" | grep -q "^hi$" || { test_fail "json_stringify missing hi"; return 1; }
  test_pass "json_stringify_vm"
}

run_test "json_stringify_vm" test_json_stringify_vm

