#!/bin/bash
# selfhost_min_json_header_pipeline_v2_vm.sh — Ensure --pipeline-v2 emits non-empty header (Rust VM, child path)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi

test_selfhost_min_json_header_pipeline_v2_vm() {
  # Run parent runner (Rust VM) with selfhost child pipeline v2; emit-only and quiet
  local out
  out=$(NYASH_DISABLE_PLUGINS=1 \
        NYASH_USE_NY_COMPILER=1 \
        NYASH_NY_COMPILER_MIN_JSON=1 \
        NYASH_NY_COMPILER_EMIT_ONLY=1 \
        NYASH_NY_COMPILER_SKIP_PY=1 NYASH_NY_COMPILER_TIMEOUT_MS=8000 \
        NYASH_NY_COMPILER_CHILD_ARGS="--pipeline-v2" \
        NYASH_JSON_ONLY=1 NYASH_QUIET=1 \
        "$NYASH_BIN" --backend vm "$NYASH_ROOT/apps/examples/string_p0.hako" 2>/dev/null | tr -d '\r' | head -n 1)

  # Expect header to contain version/kind keys
  echo "$out" | grep -q '"version"' || { log_error "missing version in header (pipeline_v2)"; return 1; }
  echo "$out" | grep -q '"kind"'    || { log_error "missing kind in header (pipeline_v2)"; return 1; }
  echo "$out" | grep -q '"kind":"Program"' || { log_error "unexpected kind (want Program): $out"; return 1; }
  return 0
}

run_test "selfhost_min_json_header_pipeline_v2_vm" test_selfhost_min_json_header_pipeline_v2_vm || exit 1
exit 0
