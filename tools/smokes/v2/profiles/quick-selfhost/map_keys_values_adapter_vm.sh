#!/bin/bash
# map_keys_values_adapter_vm.sh — Adapter path keysA/valuesA should return arrays (nyvm/selfhost path)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

test_map_keys_values_adapter_vm() {
  # quick-selfhost defaults to plugins OFF; keys/values require plugin
  if [ "${NYASH_DISABLE_PLUGINS:-1}" = "1" ]; then
    test_skip "map_keys_values_adapter_vm" "plugins disabled"
    return 0
  fi
  local code='
static box Main { main() {
  local m = new MapBox()
  m.set("b", 2)
  m.set("a", 1)
  local ks = m.keys()
  local vs = m.values()
  print("K:" + ks.size())
  print("V:" + vs.size())
  return 0
}}
'
  out=$(run_nyash_vm -c "$code")
  k=$(echo "$out" | awk -F: '/^K:/{print $2; exit}')
  v=$(echo "$out" | awk -F: '/^V:/{print $2; exit}')
  if [ "$k" = "2" ] && [ "$v" = "2" ]; then
    test_pass "map_keys_values_adapter_vm"
  else
    echo "$out"; test_fail "expected K:2 V:2"; return 1
  fi
}

run_test "map_keys_values_adapter_vm" test_map_keys_values_adapter_vm
