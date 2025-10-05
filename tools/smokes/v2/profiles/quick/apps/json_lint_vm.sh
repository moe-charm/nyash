#!/bin/bash
# json_lint_vm.sh — Example app: JSON lint (VM)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

# Always-on: CallResolver and prelude handling make this stable in quick profile

APP_DIR="$NYASH_ROOT/apps/examples/json_lint"
# Strict mode: do not tolerate Void in VM (policy: tests must not rely on NYASH_VM_TOLERATE_VOID)
# Drop trailing VM result summary lines to keep output stable
output=$(run_nyash_vm "$APP_DIR/main.nyash" --dev | grep -v '^Result: ')

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
