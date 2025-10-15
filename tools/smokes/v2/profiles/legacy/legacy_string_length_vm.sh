#!/bin/bash
# legacy_string_length_vm.sh — Deprecated String.length() behaviour (legacy)

source "$(dirname "$0")/../../lib/test_runner.sh"
export NYASH_DISABLE_PLUGINS=1
export HAKO_PLUGIN_POLICY=off
require_env || exit 2

test_legacy_string_length_vm() {
  local code=$'static box Main {\n  main() {\n    local s = "Nyash"\n    print("" + s.length())\n    return 0\n  }\n}\n'
  local out; out=$(run_nyash_vm -c "$code" | tail -n 1 | tr -d '\r')
  # Legacy expected: 5 (length of literal)
  compare_outputs "5" "$out" "legacy_string_length_vm" || return 1
  return 0
}

run_test "legacy_string_length_vm" test_legacy_string_length_vm

