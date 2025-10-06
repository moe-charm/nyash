#!/bin/bash
# selfhost_pipeline_v2_driver_min_json_vm.sh — Direct Ny driver for pipeline v2 (emit-only) prints header

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Gate: enable explicitly to avoid dev-only driver flakiness
if [ "${SMOKES_ENABLE_PIPELINE_V2_DRIVER:-0}" != "1" ]; then
  test_skip "selfhost_pipeline_v2_driver_min_json_vm" "enable with SMOKES_ENABLE_PIPELINE_V2_DRIVER=1"
  exit 0
fi

test_selfhost_pipeline_v2_driver_min_json_vm() {
  local out
  out=$(NYASH_DISABLE_PLUGINS=1 \
        NYASH_VM_TOLERATE_VOID=1 \
        NYASH_USING=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING_AST=1 \
        NYASH_JSON_ONLY=1 NYASH_QUIET=1 \
        "$NYASH_BIN" --backend vm "$NYASH_ROOT/apps/dev/pipeline_v2_min_json.nyash" 2>/dev/null | tr -d '\r' | head -n 1)

  echo "$out" | grep -q '"version"' || { log_error "missing version in header (driver pipeline_v2)"; return 1; }
  echo "$out" | grep -q '"kind"'    || { log_error "missing kind in header (driver pipeline_v2)"; return 1; }
  echo "$out" | grep -q '"kind":"Program"' || { log_error "unexpected kind (want Program): $out"; return 1; }
  return 0
}

run_test "selfhost_pipeline_v2_driver_min_json_vm" test_selfhost_pipeline_v2_driver_min_json_vm || exit 1
exit 0
