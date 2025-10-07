#!/bin/bash
# selfhost_pipeline_namespace_with_usings_vm.sh — PipelineV2 name resolution via UsingResolverBox

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export SMOKES_DEV_LOG=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
export NYASH_USING_AST=1
export NYASH_USING=1
# Keep AST prelude merge enabled for this smoke.
# Note: SMOKES_CLEAN_ENV would unset NYASH_USING_AST in test_runner.
# Disable env cleaning here to exercise with_usings end-to-end.
export SMOKES_CLEAN_ENV=0
export NYASH_USING_DIR_NS=1
export NYASH_VM_AUTO_REGISTER_DIR_NS=1
require_env || exit 2
preflight_plugins || exit 2

TEST_main() {
  # Stage-1 JSON with Return(Call name:"Timer.now_ms", args:[Int 1])
  local ast_raw='{"version":0,"kind":"Program","body":[{"type":"Return","expr":{"type":"Call","name":"Timer.now_ms","args":[{"type":"Int","value":1}]}}]}'
  local us_raw='[{"name":"Timer"}]'  # parser形：nameのみ
  local mods_raw='{ "selfhost.core.timer": "apps/core/timer/TimerBox.hako" }'

  # Escape as Nyash string literals using Python json.dumps (backslash quoting)
  local ast=$(printf '%s' "$ast_raw" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')
  local us=$(printf '%s' "$us_raw" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')
  local mods=$(printf '%s' "$mods_raw" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')

  local tmpfile="/tmp/selfhost_with_usings_$$.nyash"
  cat > "$tmpfile" <<EOF
using "apps/selfhost-compiler/pipeline_v2/pipeline.hako" as PipelineV2

static box Main {
  main() {
    local ast = ${ast}
    local us = ${us}
    local mods = ${mods}
    local out = PipelineV2.lower_stage1_to_mir_with_usings(ast, 0, us, mods)
    print(out)
    return 0
  }
}
EOF

  local out
  out=$(run_nyash_vm "$tmpfile" 2>&1 | filter_noise)
  rm -f "$tmpfile"
  echo "$out" | grep -q 'selfhost.core.timer.now_ms' || { echo "$out"; return 1; }
  return 0
}

run_test "selfhost_pipeline_namespace_with_usings_vm" TEST_main
