#!/bin/bash
# plugin_first_noop_vm.sh - Plugin-first flags should be no-op without plugins

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_plugin_first_noop_array() {
  # Even with plugin-first flags on, without a plugin-backed ArrayBox
  # the VM must continue to use builtin handlers and produce correct results.
  local script='
  local a
  a = []
  a.push(1)
  a.push(2)
  print(a.size())
  '
  local output
  output=$(NYASH_VM_BOXCALL_PLUGIN_FIRST=1 NYASH_VM_PLUGIN_PREFER_ARRAY=1 run_nyash_vm -c "$script" 2>&1)
  check_exact "2" "$output" "plugin_first_noop_array"
}

run_test "plugin_first_noop_array" test_plugin_first_noop_array

