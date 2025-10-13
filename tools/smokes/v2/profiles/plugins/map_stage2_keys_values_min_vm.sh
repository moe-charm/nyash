#!/usr/bin/env bash
# map_stage2_keys_values_min_vm.sh — plugins: Stage-2 keys/values minimal sanity

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || true

test_map_stage2_keys_values_min_vm() {
  local code=$'static box Main {\n  main() {\n    local m = new MapBox()\n    m.set("a", 1)\n    m.set("b", 2)\n    local ks = m.keys()\n    local vs = m.values()\n    if ks.size() != 2 { return 251 }\n    if vs.size() != 2 { return 252 }\n    return 0\n  }\n}\n'
  NYASH_PLUGIN_MAP_ARRAY_HANDLE=1 out=$(run_nyash_vm -c "$code" 2>&1 | tr -d '\r')
  if echo "$out" | grep -q "extern calls disabled"; then
    test_skip "Extern disabled in this config (plugin-only)"; return 0
  fi
  out=$(printf "%s\n" "$out" | tail -n 1)
  if [ "$out" = "OK" ] || [ -z "$out" ]; then
    test_pass map_stage2_keys_values_min_vm
  else
    echo "$out"; test_fail "unexpected output"; return 1
  fi
}

run_test map_stage2_keys_values_min_vm test_map_stage2_keys_values_min_vm
