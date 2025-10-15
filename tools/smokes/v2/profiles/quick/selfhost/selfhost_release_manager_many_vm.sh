#!/bin/bash
# selfhost_release_manager_many_vm.sh — ReleaseManagerBox.release_many basic smoke

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi

TEST_main() {
  local program='
using "selfhost/vm/boxes/release_manager.hako" as Release

static box Main {
  make_array() {
    local a = new ArrayBox()
    // Create few ArrayBox items (non-identity) and some StringBox; release should be no-op and safe
    a.push(new ArrayBox())
    a.push(new StringBox("x"))
    a.push(new ArrayBox())
    return a
  }
  main() {
    local objs = me.make_array()
    Release.release_many(objs)
    print("OK")
    return 0
  }
}
'
  local out
  out=$(run_nyash_vm -c "$program" 2>&1 | filter_noise)
  echo "$out" | grep -q '^OK$' || { echo "$out"; return 1; }
  return 0
}

run_test "selfhost_release_manager_many_vm" TEST_main
