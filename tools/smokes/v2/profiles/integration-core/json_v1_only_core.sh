#!/bin/bash
# json_v1_only_core.sh - Ensure JSON emitter uses unified v1 (mir_call) only

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

preflight_plugins || true  # not required

test_json_v1_only() {
  local code='print("ok")'
  local tmpjson="tmp/nyash_harness_mir.json"
  rm -f "$tmpjson" || true
  local tmpcode
  tmpcode=$(mktemp /tmp/json_v1_only_core_XXXX.hako)
  echo "$code" > "$tmpcode"
  if ! PYTHONPATH="${PYTHONPATH:-$NYASH_ROOT}" \
       NYASH_NY_LLVM_COMPILER="${NYASH_NY_LLVM_COMPILER:-$NYASH_ROOT/target/release/ny-llvmc}" \
        NYASH_EMIT_EXE_NYRT="${NYASH_EMIT_EXE_NYRT:-$NYASH_ROOT/target/release}" \
       NYASH_LLVM_USE_HARNESS=1 NYASH_DISABLE_PLUGINS=1 \
       ./target/release/nyash --emit-mir-json "$tmpjson" --backend llvm "$tmpcode" --dev >/dev/null 2>&1; then
    echo "llvm harness run failed" >&2
    return 1
  fi
  [ -f "$tmpjson" ] || { echo "missing MIR json: $tmpjson" >&2; return 1; }
  # Require presence of v1 mir_call and absence of legacy op:call
  grep -q '"mir_call"' "$tmpjson" || { echo "missing mir_call v1 object" >&2; return 1; }
  if grep -q '"op"[[:space:]]*:[[:space:]]*"call"' "$tmpjson"; then
    echo "found legacy call op in JSON" >&2
    return 1
  fi
  return 0
}

run_test "json_v1_only_core" test_json_v1_only

