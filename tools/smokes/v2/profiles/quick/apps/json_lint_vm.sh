#!/bin/bash
# json_lint_vm.sh — Example app: JSON lint (VM)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

APP_DIR="$NYASH_ROOT/apps/examples/json_lint"
export NYASH_VM_TOLERATE_VOID=1
output=$(run_nyash_vm "$APP_DIR/main.nyash" --dev)

expected=$(cat << 'TXT'
OK
OK
OK
OK
OK
OK
OK
OK
OK
OK
ERROR
ERROR
ERROR
ERROR
ERROR
ERROR
TXT
)

compare_outputs "$expected" "$output" "json_lint_vm" || exit 1
