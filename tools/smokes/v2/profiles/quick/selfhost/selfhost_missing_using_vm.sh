#!/bin/bash
# selfhost_missing_using_vm.sh — Expect strict error when using alias is missing

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

TEST_main() {
  # Stage-1 JSON with Return(Call name:"Timer.now", args:[Int 1]) but no using provided
  local ast_raw='{"version":0,"kind":"Program","body":[{"type":"Return","expr":{"type":"Call","name":"Timer.now","args":[{"type":"Int","value":1}]}}]}'
  local us_raw='[]'  # intentionally empty
  local mods_raw='{ "selfhost.core.timer": "apps/core/timer/TimerBox.hako" }'

  # Escape literals as Nyash strings using Python json.dumps
  local ast=$(printf '%s' "$ast_raw" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')
  local us=$(printf '%s' "$us_raw" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')
  local mods=$(printf '%s' "$mods_raw" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')

  local program='
using "apps/selfhost-compiler/pipeline_v2/namespace_box.hako" as NamespaceBox
using "apps/selfhost-compiler/pipeline_v2/using_resolver_box.hako" as UsingResolverBox

static box Main {
  main() {
    // Build empty resolver (no usings)
    local r = new UsingResolverBox()
    // This must fail strictly due to missing using alias for "Timer"
    local fq = NamespaceBox.normalize_global_name("Timer.now", r)
    if fq != null { print(fq) }
    return 0
  }
}
'
  # No substitutions required for the simplified test

  local out
  out=$(run_nyash_vm -c "$program" 2>&1 | filter_noise)
  echo "$out" | grep -Eq "\[ERROR\] Unresolved using alias: Timer|Key not found: Timer\.now" || { echo "$out"; return 1; }
  return 0
}

run_test "selfhost_missing_using_vm" TEST_main
