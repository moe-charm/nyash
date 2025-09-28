#!/bin/bash
# dev_selfhost_min_json_head.sh — Selfhost compiler emits non-empty JSON head (dev-gated)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Gate: run only when explicitly enabled by env to avoid noise
if [ "${SMOKES_ENABLE_SELFHOST_ACCEPT:-0}" != "1" ]; then
  test_skip "dev_selfhost_min_json_head" "enable with SMOKES_ENABLE_SELFHOST_ACCEPT=1"
  exit 0
fi

OUT=$(NYASH_DISABLE_PLUGINS=1 NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 NYASH_ALLOW_USING_FILE=1 NYASH_ENABLE_USING=1 NYASH_JSON_ONLY=1 \
      timeout 5 "$NYASH_BIN" --backend vm "$NYASH_ROOT/apps/selfhost-compiler/compiler.nyash" -- --min-json 2>/dev/null | \
      awk 'match($0,/^\{/) {print; exit}')

if echo "$OUT" | grep -q '"version"' && echo "$OUT" | grep -q '"kind"'; then
  test_pass "dev_selfhost_min_json_head"
  exit 0
else
  test_fail "dev_selfhost_min_json_head" "no JSON head"
  exit 1
fi

