#!/bin/bash
# strict_plugin_array_unknown_method_vm.sh — Strict policy: Array unknown method surfaces plugin error (no builtin fallback)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_strict_plugin_array_unknown_method_vm() {
  local code=$'static box Main {\n  main() {\n    local a = new ArrayBox()\n    a.push(1)\n    // Call a non-existent method to ensure no fallback to builtin in strict mode\n    print(a.noSuchMethod())\n    return 0\n  }\n}'
  local out
  out=$(HAKO_PLUGIN_POLICY=force run_nyash_vm -c "$code" | filter_noise)
  if echo "$out" | grep -q '^SMOKES_ERR: invalid_inst Plugin method ArrayBox.noSuchMethod failed'; then
    test_pass strict_plugin_array_unknown_method_vm
  else
    compare_outputs 'SMOKES_ERR: invalid_inst Plugin method ArrayBox.noSuchMethod failed' "$out" strict_plugin_array_unknown_method_vm
  fi
}

run_test strict_plugin_array_unknown_method_vm test_strict_plugin_array_unknown_method_vm

