#!/bin/bash
# json_pp_vm.sh — Example app: JSON pretty printer (VM)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

APP_DIR="$NYASH_ROOT/apps/examples/json_pp"
# Tolerate Void in comparisons during dev hardening (must be set before run)
export NYASH_VM_TOLERATE_VOID=1
output=$(run_nyash_vm "$APP_DIR/main.nyash" --dev)

expected=$(cat << 'TXT'
null
true
false
42
"hello"
[]
{}
{"a":1}
0
0
3.14
-2.5
6.02e23
-1e-9
TXT
)

compare_outputs "$expected" "$output" "json_pp_vm" || exit 1
