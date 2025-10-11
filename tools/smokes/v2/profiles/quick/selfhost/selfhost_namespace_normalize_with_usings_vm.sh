#!/bin/bash
# selfhost_namespace_normalize_with_usings_vm.sh — UsingResolverBox + NamespaceBox normalization only (no MIR build)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

TEST_main() {
  # usings: Timer (ns), modules: selfhost.core.timer → some path (dummy path is fine for normalization)
  local us_raw='[{"name":"Timer"}]'
  local mods_raw='{ "selfhost.core.timer": "apps/core/timer/TimerBox.hako" }'

  # Escape Nyash strings
  local us=$(printf '%s' "$us_raw" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')
  local mods=$(printf '%s' "$mods_raw" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')

  local program='
using "selfhost/compiler/pipeline_v2/using_resolver_box.hako" as UsingResolverBox
using "selfhost/compiler/pipeline_v2/namespace_box.hako" as NamespaceBox

static box Main {
  main() {
    local r = new UsingResolverBox()
    r.load_usings_json(__US__)
    r.load_modules_json(__MODS__)
    // Provide alias→namespace explicitly for this smoke
    r.add_ns("Timer", "selfhost.core.timer")
    local name = NamespaceBox.normalize_global_name("Timer.now", r)
    print(name)
    return 0
  }
}
'
  program=${program/__US__/$us}
  program=${program/__MODS__/$mods}

  local out
  out=$(run_nyash_vm -c "$program" 2>&1 | filter_noise | tail -n 1 | tr -d '
' | xargs)
  [ "$out" = "selfhost.core.timer.now" ] || { echo "$out"; return 1; }
  return 0
}

run_test "selfhost_namespace_normalize_with_usings_vm" TEST_main
