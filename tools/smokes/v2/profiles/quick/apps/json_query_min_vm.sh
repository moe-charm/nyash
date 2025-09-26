#!/bin/bash
# json_query_min_vm.sh — Minimal JSON query (VM)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

APP_DIR="$NYASH_ROOT/apps/examples/json_query_min"
export NYASH_ALLOW_USING_FILE=1
output=$(run_nyash_vm "$APP_DIR/main.nyash" --dev)

expected=$(cat << 'TXT'
2
TXT
)

compare_outputs "$expected" "$output" "json_query_min_vm" || exit 1
