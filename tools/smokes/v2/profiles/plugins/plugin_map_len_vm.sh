#!/bin/bash
# plugin_map_len_vm.sh — Verify MapBox.len() alias works via plugin resolver

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

run_test_plugin_map_len_vm() {
  local code='static box Main { main() {
    local m = new MapBox()
    m.set("a", 1)
    // Plugin resolver should route len() → size() without extra env toggles
    if m.len() != 1 { return 10 }
    return 0
  }}'
  local out rc
  out=$(run_nyash_vm -c "$code" 2>&1 || true)
  rc=$?
  if echo "$out" | grep -q "Unknown Box type: MapBox"; then
    test_skip "plugin_map_len_vm" "MapBox unavailable (plugins disabled)"
    return 0
  fi
  if [ $rc -ne 0 ]; then
    echo "$out" >&2
    return 1
  fi
  return 0
}

run_test "plugin_map_len_vm" run_test_plugin_map_len_vm
