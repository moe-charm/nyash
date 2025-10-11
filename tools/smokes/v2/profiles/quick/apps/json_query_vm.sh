#!/bin/bash
# json_query_vm.sh — Example app: JSON query (VM)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

## Enabled: final guard applied

APP_DIR="$NYASH_ROOT/apps/examples/json_query"
# Use default dev behavior (rewrite enabled) for stable resolution
# NOTE: Do not enable NYASH_VM_TOLERATE_VOID here; path parser relies on strict compare semantics
run_nyash_vm "$APP_DIR/main.nyash" --dev >/dev/null
rc=$?
if [ $rc -ne 0 ]; then
  echo "FAIL: rc=$rc" >&2
  exit 1
fi
echo "OK"
