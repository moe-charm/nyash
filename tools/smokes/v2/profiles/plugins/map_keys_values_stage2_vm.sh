#!/bin/bash
# map_keys_values_stage2_vm.sh — Plugins: Map.keys/values HostHandle (Stage-2) path

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

# Stage-2 HostHandle path requires explicit opt-in
export NYASH_PLUGIN_MAP_ARRAY_HANDLE=1
export NYASH_USE_PLUGIN_BUILTINS=${NYASH_USE_PLUGIN_BUILTINS:-1}
export NYASH_PLUGIN_OVERRIDE_TYPES=${NYASH_PLUGIN_OVERRIDE_TYPES:-"StringBox,ArrayBox,MapBox"}
export NYASH_BUILTIN_DISABLE_STRING=${NYASH_BUILTIN_DISABLE_STRING:-1}
export NYASH_BUILTIN_DISABLE_ARRAY=${NYASH_BUILTIN_DISABLE_ARRAY:-1}
export NYASH_BUILTIN_DISABLE_MAP=${NYASH_BUILTIN_DISABLE_MAP:-1}

run_test_map_stage2() {
  local code='static box Main { main() {
    local m = new MapBox()
    local ks0 = m.keys()
    if ks0.size() != 0 { return 10 }
    m.set("a", 1)
    m.set("b", 2)
    local ks = m.keys()
    if ks.size() != 2 { return 11 }
    if ks.get(0) != "a" || ks.get(1) != "b" { return 12 }
    local vs = m.values()
    if vs.size() != 2 { return 13 }
    if vs.get(0) != 1 || vs.get(1) != 2 { return 14 }
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code"); rc=$?
  if [ $rc -ne 0 ]; then
    echo "FAIL: non-zero exit ($rc)"
    return 1
  fi
  return 0
}

run_test "map_keys_values_stage2_vm" run_test_map_stage2
