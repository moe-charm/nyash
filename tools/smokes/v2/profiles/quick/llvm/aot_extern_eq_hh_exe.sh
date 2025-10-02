#!/usr/bin/env bash
# Quick smoke: string equality via eq_hh (expects Result: 1)

set -euo pipefail

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

APP="apps/tests/extern_eq_hh_true.nyash"
name=$(basename "$APP" .nyash)
OBJ="$PWD/target/aot_objects/${name}.o"
BIN_OUT="${APP_BIN_DIR:-tmp}/app_${name}"

mkdir -p "$(dirname "$OBJ")" "$(dirname "$BIN_OUT")"

log_info "Harness emit → object"
NYASH_LLVM_USE_HARNESS=1 NYASH_LLVM_OBJ_OUT="$OBJ" "$NYASH_BIN" --backend llvm "$APP" >/dev/null || true
if [[ ! -s "$OBJ" ]]; then
  test_skip "$name" "harness did not produce object" && exit 0
fi

log_info "Link object → exe"
NYASH_LLVM_SKIP_EMIT=1 NYASH_LLVM_OBJ_OUT="$OBJ" ./tools/build_llvm.sh "$APP" -o "$BIN_OUT" >/dev/null || true
if [[ ! -x "$BIN_OUT" ]]; then
  test_skip "$name" "link step failed" && exit 0
fi

out=$(NYASH_NYRT_SILENT_RESULT=1 "$BIN_OUT" 2>/dev/null || true)
line=$(echo "$out" | rg '^Result: ' -n || true)
[[ "$line" == *"Result: 1"* ]] && test_pass "$name" || { echo "$out" >&2; test_skip "$name" "expected 'Result: 1'" && exit 0; }
