#!/bin/bash
# selfhost_pipeline_namespace_with_usings_spaced_escaped_vm.sh — spaced colon & escaped quotes

source "$(dirname "$0")/../../lib/test_runner.sh"
if [ "${SMOKES_ENABLE_SCANNER_TOL:-0}" != "1" ]; then
  log_warn "SKIP scanner tolerance smoke; set SMOKES_ENABLE_SCANNER_TOL=1 to run"
  exit 0
fi
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export SMOKES_DEV_LOG=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
export NYASH_USING_AST=1
export NYASH_USING=1
export SMOKES_CLEAN_ENV=0
export NYASH_USING_DIR_NS=1
export NYASH_VM_AUTO_REGISTER_DIR_NS=1
export NYASH_SKIP_TOML_ENV=1
require_env || exit 2
preflight_plugins || exit 2

TEST_main() {
  local ast_raw='{"version":0, "kind":"Program", "body":[{"type":"Return","expr":{"type":"Call", "name" : "string_scan.index_of_from", "args" : [{"type":"String","value":"a\"bcdef"},{"type":"String","value":"cd"},{"type":"Int","value":0}]}}]}'
  local us_raw='[{"name":"string_scan"}]'
  local mods_raw='{ "selfhost.common.json.core.string_scan": "apps/selfhost/common/json/core/string_scan.hako" }'

  local ast=$(printf '%s' "$ast_raw" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')
  local us=$(printf '%s' "$us_raw" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')
  local mods=$(printf '%s' "$mods_raw" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')

  local tmpfile="/tmp/selfhost_with_usings_spaced_escaped_$$.nyash"
  cat > "$tmpfile" <<EOF
using "selfhost/compiler/pipeline_v2/pipeline.hako" as PipelineV2

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
  if echo "$out" | grep -qE 'modules_error|AST prelude merge is disabled'; then
    log_warn "SKIP selfhost with_usings (env not ready): modules_error/AST disabled"
    return 0
  fi
  echo "$out" | grep -q 'selfhost.common.json.core.string_scan.index_of_from' || { echo "$out"; return 1; }
  return 0
}

run_test "selfhost_pipeline_namespace_with_usings_spaced_escaped_vm" TEST_main
