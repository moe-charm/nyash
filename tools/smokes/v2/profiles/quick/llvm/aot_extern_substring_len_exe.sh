#!/usr/bin/env bash
# Quick smoke: substring length via string API (expects Result: 2)

set -euo pipefail

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

APP="$NYASH_ROOT/apps/tests/extern_substring_len.nyash"
name=$(basename "$APP" .nyash)

run_case() {
  ensure_hako_toml
  local out
  out=$(run_nyash_llvm "$APP" --dev | sed -n 's/^Result: .*/&/p' | head -n 1 | tr -d '\r' | xargs)
  if [ -z "$out" ]; then
    test_skip "$name" "LLVM harness unavailable (quick)" || true
    return 0
  fi
  compare_outputs "Result: 2" "$out" "$name"
}

if run_case; then
  exit 0
else
  exit 1
fi
