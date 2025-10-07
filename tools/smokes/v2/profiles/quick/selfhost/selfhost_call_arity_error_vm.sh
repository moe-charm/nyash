#!/bin/bash
# selfhost_call_arity_error_vm.sh — Call arity verification for built-in-like names

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_ENABLE_CALL_ARITY:-0}" != "1" ]; then
  log_warn "SKIP selfhost_call_arity_error_vm (enable with SMOKES_ENABLE_CALL_ARITY=1)"
  exit 0
fi

TEST_main() {
  # Stage-1 JSON: Return(Call name:"StringBox.indexOf", args:[Int 1, Int 2])
  local ast_raw='{"version":0,"kind":"Program","body":[{"type":"Return","expr":{"type":"Call","name":"StringBox.indexOf","args":[{"type":"Int","value":1},{"type":"Int","value":2}]}}]}'
  local ast=$(printf '%s' "$ast_raw" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')

  local program='
using "apps/selfhost-compiler/pipeline_v2/pipeline.hako" as PipelineV2
static box Main {
  main(){
    local ast = __AST__
    // prefer ret path
    local out = PipelineV2.lower_stage1_to_mir(ast, 0)
    if out != null { print(out) }
    return 0
  }
}
'
  program=${program/__AST__/$ast}
  local out
  out=$(run_nyash_vm -c "$program" 2>&1 | filter_noise)
  echo "$out" | grep -q "\[ERROR\] No matching call by arity: StringBox.indexOf/2" || { echo "$out"; return 1; }
  return 0
}

run_test "selfhost_call_arity_error_vm" TEST_main
