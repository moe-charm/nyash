#!/bin/bash
# plugin_on_print_array_size_vm.sh — ensure print(arr.size()) does not hang

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

export NYASH_PLUGIN_MAP_ARRAY_HANDLE=1
export NYASH_USE_PLUGIN_BUILTINS=${NYASH_USE_PLUGIN_BUILTINS:-1}
export NYASH_PLUGIN_OVERRIDE_TYPES=${NYASH_PLUGIN_OVERRIDE_TYPES:-"StringBox,ArrayBox,MapBox"}
export NYASH_BUILTIN_DISABLE_STRING=${NYASH_BUILTIN_DISABLE_STRING:-1}
export NYASH_BUILTIN_DISABLE_ARRAY=${NYASH_BUILTIN_DISABLE_ARRAY:-1}
export NYASH_BUILTIN_DISABLE_MAP=${NYASH_BUILTIN_DISABLE_MAP:-1}

run_test_plugin_on_print_array_size_vm() {
  local code=$'static box Main {\n  main() {\n    local arr = new ArrayBox()\n    arr.push(1)\n    print(arr.size())\n    return 0\n  }\n}\n'
  # Use timeout to avoid hanging forever; expect to finish with exit 0 and print "1"
  local out
  out=$(SMOKES_TIMEOUT_SEC=10 run_nyash_vm -c "$code" --dev)
  if ! echo "$out" | grep -q '^1$'; then
    echo "FAIL: expected to print 1, got:"; echo "$out" | tail -n +1
    return 1
  fi
  if ! echo "$out" | grep -q '^Result: 0$'; then
    echo "FAIL: result not 0"; echo "$out" | tail -n +1
    return 1
  fi
  return 0
}

run_test "plugin_on_print_array_size_vm" run_test_plugin_on_print_array_size_vm
