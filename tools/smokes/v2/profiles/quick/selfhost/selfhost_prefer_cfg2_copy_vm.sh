#!/bin/bash
# selfhost_prefer_cfg2_copy_vm.sh — ensure prefer-cfg2 inserts a copy op (materialize) in MIR(JSON)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Experimental guard: run only when explicitly enabled
if [[ "${SMOKES_ENABLE_CFG2_SMOKE:-}" != "1" ]]; then
  test_skip "prefer-cfg2 smoke is experimental; set SMOKES_ENABLE_CFG2_SMOKE=1 to enable"
  exit 0
fi

test_selfhost_prefer_cfg2_copy_vm() {
  local json
  json=$(NYASH_DISABLE_PLUGINS=1 \
         NYASH_ENABLE_USING=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING_AST=1 \
         NYASH_PREFER_CFG2=1 NYASH_JSON_ONLY=1 \
         "$NYASH_BIN" --backend vm "$NYASH_ROOT/apps/selfhost-compiler/compiler.hako" -- --min-json --pipeline-v2 --emit-mir 2>/dev/null | tr -d '\r' | awk 'match($0,/^\{.*\}$/){line=$0} END{print line}')
  echo "$json" | grep -q '"op":"copy"' || { log_error "missing materialize copy in MIR(JSON)"; return 1; }
  return 0
}

run_test "selfhost_prefer_cfg2_copy_vm" test_selfhost_prefer_cfg2_copy_vm || exit 1
exit 0
