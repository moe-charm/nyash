#!/bin/bash
# using_auto_dir_namespace_vm.sh — Auto directory-as-namespace resolver E2E (dev-only)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
# Ensure auto-dir fallback is ON even outside run.sh overlay
export NYASH_USING_DIR_NS=1
require_env || exit 2
preflight_plugins || exit 2

TEST_main() {
  local program='
using selfhost.core.timer as TimerBox

static box Main {
  main() {
    // Call a simple static method to ensure resolution worked
    local t = TimerBox.now_ms()
    // Print a stable token for the test to assert
    print("AUTO_DIR_OK")
    return 0
  }
}
'
  local out
  out=$(run_nyash_vm -c "$program")
  echo "$out" | grep -q 'AUTO_DIR_OK' || { echo "$out"; return 1; }
  return 0
}

run_test "using_auto_dir_namespace_vm" TEST_main
