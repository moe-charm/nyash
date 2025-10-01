#!/bin/bash
# selfhost_emit_trace_pipeline_v2_vm.sh — detect dev trace line and final JSON when --emit-trace + pipeline_v2 + emit-mir

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

test_selfhost_emit_trace_pipeline_v2_vm() {
  local out all first json
  # Parent→child: pass pipeline-v2 + emit-mir + emit-trace; ensure child prints one [emit] line and final JSON
  all=$(NYASH_DISABLE_PLUGINS=1 \
        NYASH_ENABLE_USING=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING_AST=1 \
        NYASH_EMIT_TRACE=1 NYASH_JSON_ONLY=1 \
        "$NYASH_BIN" --backend vm "$NYASH_ROOT/apps/selfhost-compiler/compiler.hako" -- --min-json --pipeline-v2 --emit-mir 2>/dev/null | tr -d '\r')

  # first non-empty line should be a trace
  first=$(echo "$all" | awk 'NF {print; exit}')
  echo "$first" | grep -q '^\[emit\] ' || { log_error "missing [emit] first line"; echo "$all" | tail -n 20 >&2; return 1; }

  # last JSON line should contain version/kind
  json=$(echo "$all" | awk 'match($0,/^\{.*\}$/){line=$0} END{print line}')
  echo "$json" | grep -q '"version"' || { log_error "missing version in final JSON"; return 1; }
  echo "$json" | grep -q '"kind"' || { log_error "missing kind in final JSON"; return 1; }
  return 0
}

run_test "selfhost_emit_trace_pipeline_v2_vm" test_selfhost_emit_trace_pipeline_v2_vm || exit 1
exit 0
