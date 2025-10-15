#!/bin/bash
# map_call_boundary_mixed_vm.sh — Mixed boundary: missing key returns null; non-callable errors

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

test_map_call_boundary_mixed_vm() {
  local code_missing=$'static box Main {\n  main() {\n    local m = new MapBox()\n    local args = new ArrayBox()\n    local r = m.call("missing", args)\n    print("" + r)\n    return 0\n  }\n}\n'
  local out rc
  out=$(run_nyash_vm -c "$code_missing" 2>&1 | filter_noise | tail -n 1 | tr -d '\r')
  rc=$?
  if [ $rc -ne 0 ]; then
    log_error "unexpected non-zero rc for missing key: rc=$rc out=$out"; return 1
  fi
  if ! echo "$out" | grep -qx 'null'; then
    log_error "expected 'null' for missing key, got: $out"; return 1
  fi

  local code_noncall=$'static box Main {\n  main() {\n    local m = new MapBox()\n    m.set("x", 123)\n    local args = new ArrayBox()\n    m.call("x", args)\n    return 0\n  }\n}\n'
  run_nyash_vm -c "$code_noncall" >/dev/null 2>&1
  rc=$?
  if [ $rc -eq 0 ]; then
    log_error "expected non-zero exit for non-callable value"
    return 1
  fi
  return 0
}

run_test "map_call_boundary_mixed_vm" test_map_call_boundary_mixed_vm
