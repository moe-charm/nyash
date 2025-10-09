#!/bin/bash
# json_no_externcall_core.sh - Ensure JSON emitter never outputs legacy externcall

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

preflight_plugins || true  # not required

test_json_has_no_externcall() {
  local code='print("ok")'
  local tmpjson="tmp/nyash_harness_mir.json"
  rm -f "$tmpjson" || true
  # Trigger LLVM harness to emit MIR JSON
  # Write code to a temp file (LLVM CLI doesn't accept -c)
  local tmpcode
  tmpcode=$(mktemp /tmp/json_no_externcall_XXXX.hako)
  echo "$code" > "$tmpcode"
  if ! PYTHONPATH="${PYTHONPATH:-$NYASH_ROOT}" \
       NYASH_NY_LLVM_COMPILER="${NYASH_NY_LLVM_COMPILER:-$NYASH_ROOT/target/release/ny-llvmc}" \
       NYASH_EMIT_EXE_NYRT="${NYASH_EMIT_EXE_NYRT:-$NYASH_ROOT/target/release}" \
       NYASH_LLVM_USE_HARNESS=1 NYASH_DISABLE_PLUGINS=1 \
       ./target/release/nyash --emit-mir-json "$tmpjson" --backend llvm "$tmpcode" --dev >/dev/null 2>&1; then
    echo "llvm harness run failed" >&2
    return 1
  fi
  # Basic sanity: JSON file must exist and contain functions
  [ -f "$tmpjson" ] || { echo "missing MIR json: $tmpjson" >&2; return 1; }
  grep -q '"functions"' "$tmpjson" || { echo "no functions in JSON"; return 1; }
  # Assert no legacy externcall op is present
  if grep -q '"op"[[:space:]]*:[[:space:]]*"externcall"' "$tmpjson"; then
    echo "found legacy externcall in JSON" >&2
    return 1
  fi
  return 0
}

run_test "json_no_externcall_core" test_json_has_no_externcall
