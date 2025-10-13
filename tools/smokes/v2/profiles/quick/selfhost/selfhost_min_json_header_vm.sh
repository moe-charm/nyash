#!/bin/bash
# selfhost_min_json_header_vm.sh — Ensure selfhost (--min-json) emits non-empty header (Rust VM default)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi

test_selfhost_min_json_header_vm() {
  # Use runner with selfhost child; emit-only and quiet
  local out
  out=$(NYASH_DISABLE_PLUGINS=1 \
        NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING=1 NYASH_USING_AST=0 \
        NYASH_JSON_ONLY=0 \
        run_nyash_vm "$NYASH_ROOT/apps/selfhost-compiler/compiler.hako" -- --min-json | \
        awk 'match($0,/^\{/) {print; exit}')

  # Expect header to contain version/kind keys
  echo "$out" | grep -q '"version"' || { log_error "missing version in header"; return 1; }
  echo "$out" | grep -q '"kind"'    || { log_error "missing kind in header"; return 1; }
  # And kind should be Program in minimal AST JSON
  echo "$out" | grep -q '"kind":"Program"' || { log_error "unexpected kind (want Program): $out"; return 1; }
  return 0
}

run_test "selfhost_min_json_header_vm" test_selfhost_min_json_header_vm || exit 1
exit 0
