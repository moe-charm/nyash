#!/bin/bash
# map_stage2_identity_vm.sh — Stage-2 HostHandle identity for Map.values()

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

export NYASH_PLUGIN_MAP_ARRAY_HANDLE=1
export NYASH_USE_PLUGIN_BUILTINS=${NYASH_USE_PLUGIN_BUILTINS:-1}
export NYASH_PLUGIN_OVERRIDE_TYPES=${NYASH_PLUGIN_OVERRIDE_TYPES:-"StringBox,ArrayBox,MapBox"}
export NYASH_BUILTIN_DISABLE_STRING=${NYASH_BUILTIN_DISABLE_STRING:-1}
export NYASH_BUILTIN_DISABLE_ARRAY=${NYASH_BUILTIN_DISABLE_ARRAY:-1}
export NYASH_BUILTIN_DISABLE_MAP=${NYASH_BUILTIN_DISABLE_MAP:-1}

run_test_map_stage2_identity() {
  local code='static box Main { main() {
    local arr = new ArrayBox()
    arr.push(1)
    local m = new MapBox()
    m.set("list", arr)
    local vals = m.values()
    if vals.size() != 1 { return 20 }
    local first = vals.get(0)
    first.push(2)
    local vals2 = m.values()
    if vals2.size() != 1 { return 21 }
    local again = vals2.get(0)
    if again.size() != 2 { return 22 }
    if again.get(0) != 1 || again.get(1) != 2 { return 23 }
    return 0
  }}'
  local out
  out=$(run_nyash_vm -c "$code"  | awk '/^Result:/{print $2}')
  if [[ "$out" != "0" ]]; then
    echo "FAIL: Result: $out"
    return 1
  fi
  return 0
}

run_test "map_stage2_identity_vm" run_test_map_stage2_identity
