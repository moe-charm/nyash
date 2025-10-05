#!/bin/bash
# selfhost_pipeline_namespace_with_usings_vm.sh — PipelineV2 name resolution via UsingResolverBox

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

TEST_main() {
  # Stage-1 JSON with Return(Call name:"Timer.now", args:[Int 1])
  local ast_raw='{"version":0,"kind":"Program","body":[{"type":"Return","expr":{"type":"Call","name":"Timer.now","args":[{"type":"Int","value":1}]}}]}'
  local us_raw='[{"name":"Timer"}]'  # parser形：nameのみ
  local mods_raw='{ "selfhost.core.timer": "apps/core/timer/TimerBox.hako" }'

  # Escape as Nyash string literals using Python json.dumps (backslash quoting)
  local ast=$(python3 - << 'PY'
import json,sys
print(json.dumps(sys.stdin.read()))
PY
<<< "$ast_raw")
  local us=$(python3 - << 'PY'
import json,sys
print(json.dumps(sys.stdin.read()))
PY
<<< "$us_raw")
  local mods=$(python3 - << 'PY'
import json,sys
print(json.dumps(sys.stdin.read()))
PY
<<< "$mods_raw")

  local program='
using "apps/selfhost-compiler/pipeline_v2/pipeline.hako" as PipelineV2

static box Main {
  main() {
    local ast = __AST__
    local us = __US__
    local mods = __MODS__
    local out = PipelineV2.lower_stage1_to_mir_with_usings(ast, 0, us, mods)
    print(out)
    return 0
  }
}
'
  program=${program/__AST__/$ast}
  program=${program/__US__/$us}
  program=${program/__MODS__/$mods}

  local out
  out=$(run_nyash_vm -c "$program" 2>&1 | filter_noise)
  echo "$out" | grep -q 'selfhost.core.timer.now' || { echo "$out"; return 1; }
  return 0
}

run_test "selfhost_pipeline_namespace_with_usings_vm" TEST_main
