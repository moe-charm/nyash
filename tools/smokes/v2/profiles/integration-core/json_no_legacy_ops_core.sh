#!/bin/bash
# json_no_legacy_ops_core.sh - Ensure JSON emitter never outputs legacy ops

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

preflight_plugins || true  # not required

test_json_has_no_legacy_ops() {
  local code='print("ok")'
  local tmpjson="tmp/nyash_harness_mir.json"
  rm -f "$tmpjson" || true
  local tmpcode
  tmpcode=$(mktemp /tmp/json_no_legacy_ops_XXXX.hako)
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
  grep -q '"functions"' "$tmpjson" || { echo "no functions in JSON"; return 1; }
  # Assert no legacy ops appear in JSON (externcall/array_get/array_set/ref_get/ref_set/plugin_invoke)
  if grep -Eq '"op"[[:space:]]*:[[:space:]]*"externcall"' "$tmpjson"; then echo "found legacy externcall" >&2; return 1; fi
  if grep -Eq 'array_get|array_set|ref_get|ref_set|plugin_invoke' "$tmpjson"; then echo "found legacy array/ref/plugin ops" >&2; return 1; fi
  return 0
}

run_test "json_no_legacy_ops_core" test_json_has_no_legacy_ops

