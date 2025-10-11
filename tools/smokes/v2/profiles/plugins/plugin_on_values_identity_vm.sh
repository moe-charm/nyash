#!/bin/bash
# plugin_on_values_identity_vm.sh — Stage-2 HostHandle identity via Map.values()

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

export NYASH_PLUGIN_MAP_ARRAY_HANDLE=1
export NYASH_USE_PLUGIN_BUILTINS=${NYASH_USE_PLUGIN_BUILTINS:-1}
export NYASH_PLUGIN_OVERRIDE_TYPES=${NYASH_PLUGIN_OVERRIDE_TYPES:-"StringBox,ArrayBox,MapBox"}
export NYASH_BUILTIN_DISABLE_STRING=${NYASH_BUILTIN_DISABLE_STRING:-1}
export NYASH_BUILTIN_DISABLE_ARRAY=${NYASH_BUILTIN_DISABLE_ARRAY:-1}
export NYASH_BUILTIN_DISABLE_MAP=${NYASH_BUILTIN_DISABLE_MAP:-1}

run_test_plugin_on_values_identity_vm() {
  local code=$'static box Main {\n  main() {\n    local arr = new ArrayBox()\n    arr.push(1)\n    local m = new MapBox()\n    m.set("list", arr)\n    local vals = m.values()\n    local first = vals.get(0)\n    first.push(2)\n    local vals2 = m.values()\n    local again = vals2.get(0)\n    if again.size() != 2 { print("NG"); return 22 }\n    print("OK");\n    return 0\n  }\n}\n'
  local out
  out=$(SMOKES_TIMEOUT_SEC=10 run_nyash_vm -c "$code" | tail -n 1 | tr -d '\r' | xargs)
  if [[ "$out" != "OK" ]]; then
    echo "FAIL: $out"
    return 1
  fi
  return 0
}

run_test "plugin_on_values_identity_vm" run_test_plugin_on_values_identity_vm
