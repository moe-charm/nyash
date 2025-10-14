#!/bin/bash
# strict_plugin_fallback_block_vm.sh — Strict policy blocks builtin fallback when plugin provider exists

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

test_strict_plugin_fallback_block_vm() {
  local code=$'static box Main {\n  main() {\n    local m = new MapBox()\n    // Call a non-existent method to force router fallback\n    print(m.noSuchMethod())\n    return 0\n  }\n}'
  local out
  # Strict policy forbids builtin fallback when a plugin provider exists for MapBox
  out=$(HAKO_PLUGIN_POLICY=force run_nyash_vm -c "$code" | filter_noise)
  # Under strict policy, plugin provider answers; unknown method surfaces as plugin error, not builtin fallback
  if echo "$out" | grep -q '^SMOKES_ERR: invalid_inst Plugin method MapBox.noSuchMethod failed'; then
    test_pass strict_plugin_fallback_block_vm
  else
    compare_outputs 'SMOKES_ERR: invalid_inst Plugin method MapBox.noSuchMethod failed' "$out" strict_plugin_fallback_block_vm
  fi
}

run_test strict_plugin_fallback_block_vm test_strict_plugin_fallback_block_vm
