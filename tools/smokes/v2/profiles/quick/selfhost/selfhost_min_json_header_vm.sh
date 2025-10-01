#!/bin/bash
# selfhost_min_json_header_vm.sh — Ensure selfhost (--min-json) emits non-empty header (Rust VM default)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

test_selfhost_min_json_header_vm() {
  # Use runner with selfhost child; emit-only and quiet
  local out
  out=$(NYASH_DISABLE_PLUGINS=1 \
        NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 NYASH_ALLOW_USING_FILE=1 NYASH_ENABLE_USING=1 \
        NYASH_JSON_ONLY=1 \
        timeout 5 "$NYASH_BIN" --backend vm "$NYASH_ROOT/apps/selfhost-compiler/compiler.hako" -- --min-json 2>/dev/null | \
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
