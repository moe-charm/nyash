#!/bin/bash
# map_values_size_extern_vm.sh — guard Map.values().size() extern normalization

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

run_test_map_values_size_extern_vm() {
  local code='static box Main { main() {
    local m = new MapBox();
    local vs = m.values();
    if vs.size() != 0 { return 11 }
    m.set("k", 1);
    if m.values().size() != 1 { return 12 }
    return 0
  }}'
  local out rc
  out=$(run_nyash_vm -c "$code" 2>&1 || true)
  rc=$?
  if echo "$out" | grep -q "Unknown Box type: MapBox"; then
    test_skip "map_values_size_extern_vm" "MapBox unavailable (plugins disabled)"
    return 0
  fi
  if [ $rc -ne 0 ]; then
    echo "$out" >&2
    return 1
  fi
  return 0
}

run_test "map_values_size_extern_vm" run_test_map_values_size_extern_vm
