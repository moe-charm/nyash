#!/usr/bin/env bash
# Quick smoke: string equality false (expects Result: 0)

set -euo pipefail

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

APP="apps/tests/extern_eq_hh_false.nyash"
name=$(basename "$APP" .nyash)

log_info "Harness run (no link)"
out=$(PYTHONPATH="${PYTHONPATH:-$NYASH_ROOT}" NYASH_LLVM_USE_HARNESS=1 NYASH_NYRT_SILENT_RESULT=1 "$NYASH_BIN" --backend llvm "$APP" 2>&1 || true)
line=$(echo "$out" | rg '^Result: ' -n || true)
[[ "$line" == *"Result: 0"* ]] && test_pass "$name" || { echo "$out" >&2; test_fail "$name" "expected 'Result: 0'" && exit 1; }

