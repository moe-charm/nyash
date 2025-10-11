#!/bin/bash
# selfhost_namespace_normalize_call_new_vm.sh — Normalize Call and New names via UsingResolverBox/NamespaceBox

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

TEST_main() {
  local program='
using "selfhost/compiler/pipeline_v2/using_resolver_box.hako" as UsingResolverBox
using "selfhost/compiler/pipeline_v2/namespace_box.hako" as NamespaceBox

static box Main {
  main() {
    local r = new UsingResolverBox()
    r.add_ns("Timer", "selfhost.core.timer")
    r.add_ns("VM", "selfhost.vm")
    local fname = NamespaceBox.normalize_global_name("Timer.now_ms", r)
    local cname = NamespaceBox.normalize_class_name("VM.entry", r)
    print(fname)
    print(cname)
    return 0
  }
}
'
  local out
  out=$(run_nyash_vm -c "$program" 2>&1 | filter_noise)
  echo "$out" | grep -q '^selfhost.core.timer.now_ms$' || { echo "$out"; return 1; }
  echo "$out" | grep -q '^selfhost.vm.entry$' || { echo "$out"; return 1; }
  return 0
}

run_test "selfhost_namespace_normalize_call_new_vm" TEST_main
