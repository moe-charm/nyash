#!/bin/bash
# externs_effects_roundtrip_core.sh - Ensure externs registry IO effects appear in JSON

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

preflight_plugins || true

test_externs_effects_roundtrip() {
  local code='print("ok")'
  local tmpjson="tmp/nyash_harness_mir.json"
  rm -f "$tmpjson" || true
  local tmpcode
  tmpcode=$(mktemp /tmp/externs_effects_roundtrip_XXXX.hako)
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
  # Effects must contain IO for console print (Global or Extern path)
  grep -q '"IO"' "$tmpjson" || { echo "effects IO not found" >&2; return 1; }
  return 0
}

run_test "externs_effects_roundtrip_core" test_externs_effects_roundtrip
