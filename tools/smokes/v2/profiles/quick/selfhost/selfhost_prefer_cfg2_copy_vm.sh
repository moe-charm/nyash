#!/bin/bash
# selfhost_prefer_cfg2_copy_vm.sh — ensure prefer-cfg2 inserts a copy op (materialize) in MIR(JSON)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

test_selfhost_prefer_cfg2_copy_vm() {
  local json
  json=$(NYASH_DISABLE_PLUGINS=1 \
         NYASH_USE_NY_COMPILER=1 \
         NYASH_NY_COMPILER_MIN_JSON=1 \
         NYASH_NY_COMPILER_EMIT_ONLY=1 \
         NYASH_NY_COMPILER_SKIP_PY=1 NYASH_NY_COMPILER_TIMEOUT_MS=8000 \
         NYASH_NY_COMPILER_CHILD_ARGS="--pipeline-v2 --emit-mir" \
         NYASH_PREFER_CFG2=1 \
         NYASH_JSON_ONLY=1 \
         "$NYASH_BIN" --backend vm "$NYASH_ROOT/apps/examples/string_p0.hako" 2>/dev/null | tr -d '\r' | awk 'match($0,/^\{.*\}$/){line=$0} END{print line}')
  echo "$json" | grep -q '"op":"copy"' || { log_error "missing materialize copy in MIR(JSON)"; return 1; }
  return 0
}

run_test "selfhost_prefer_cfg2_copy_vm" test_selfhost_prefer_cfg2_copy_vm || exit 1
exit 0

