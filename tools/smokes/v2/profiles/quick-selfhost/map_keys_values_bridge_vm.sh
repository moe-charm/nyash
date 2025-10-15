#!/bin/bash
# map_keys_values_bridge_vm.sh — Validate .hako HostBridge wiring for Map.keys/values via keysS/valuesS

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

test_map_keys_values_bridge_vm() {
  local code='
using "selfhost/hakorune-vm/map_keys_values_bridge.hako" as Bridge

static box Main { main() {
  local m = new MapBox()
  m.set("b", 2)
  m.set("a", 1)
  local ks = Bridge.keys_array(m)
  local vs = Bridge.values_array(m)
  print("K:" + ks.size())
  print("V:" + vs.size())
  return 0
}}
'
  out=$(run_nyash_vm -c "$code")
  k=$(echo "$out" | awk -F: '/^K:/{print $2; exit}')
  v=$(echo "$out" | awk -F: '/^V:/{print $2; exit}')
  if [ "$k" = "2" ] && [ "$v" = "2" ]; then
    test_pass "map_keys_values_bridge_vm"
  else
    echo "$out"; test_fail "expected K:2 V:2"; return 1
  fi
}

run_test "map_keys_values_bridge_vm" test_map_keys_values_bridge_vm
