#!/bin/bash
# selfhost_namespace_box_basic_vm.sh — NamespaceBox basic alias→ns normalization

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

TEST_main() {
  local program='
using "apps/selfhost-compiler/pipeline_v2/using_resolver_box.hako" as UsingResolver
using "apps/selfhost-compiler/pipeline_v2/namespace_box.hako" as NamespaceBox

static box Main {
  main() {
    local r = new UsingResolverBox()
    // ns-only entry
    r.add_ns("Timer", "selfhost.core.timer")
    local n = NamespaceBox.normalize_global_name("Timer.now", r)
    print("N=" + n)
    local c = NamespaceBox.normalize_class_name("Timer.Util", r)
    print("C=" + c)
    return 0
  }
}
'
  local out
  out=$(run_nyash_vm -c "$program" 2>&1 | filter_noise)
  echo "$out" | grep -q '^N=selfhost.core.timer.now$' || { echo "$out"; return 1; }
  echo "$out" | grep -q '^C=selfhost.core.timer.Util$' || { echo "$out"; return 1; }
  return 0
}

run_test "selfhost_namespace_box_basic_vm" TEST_main
