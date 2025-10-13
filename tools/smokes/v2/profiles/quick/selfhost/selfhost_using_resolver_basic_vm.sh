#!/bin/bash
# selfhost_using_resolver_basic_vm.sh — UsingResolverBox basic resolution

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi

TEST_main() {
  # Build a tiny program that exercises UsingResolverBox
  local program='
using "selfhost/compiler/pipeline_v2/using_resolver_box.hako" as UsingResolver

static box Main {
  main() {
    local r = new UsingResolverBox()
    // Load usings: ns + path entries
    local us = "[{\"name\":\"selfhost.vm.mir_min\"},{\"name\":\"JsonNode\",\"path\":\"apps/lib/json_native/core/node.hako\"}]"
    r.load_usings_json(us)
    // Provide explicit alias for ns and modules map for path resolution
    r.add_ns("MirVmMin", "selfhost.vm.mir_min")
    r.add_module("selfhost.vm.mir_min", "apps/selfhost/vm/boxes/mir_vm_min.hako")

    local ns = r.resolve_namespace_alias("MirVmMin")
    print("ns=" + (""+ns))
    local mp = r.resolve_module_path_from_alias("MirVmMin")
    print("mod_path=" + (""+mp))
    local p = r.resolve_path_alias("JsonNode")
    print("alias_path=" + (""+p))
    return 0
  }
}
'
  local out
  out=$(run_nyash_vm -c "$program" 2>&1 | filter_noise)
  echo "$out" | grep -q '^ns=selfhost.vm.mir_min$' || { echo "$out"; return 1; }
  echo "$out" | grep -q '^mod_path=apps/selfhost/vm/boxes/mir_vm_min.hako$' || { echo "$out"; return 1; }
  echo "$out" | grep -q '^alias_path=apps/lib/json_native/core/node.hako$' || { echo "$out"; return 1; }
  return 0
}

run_test "selfhost_using_resolver_basic_vm" TEST_main
