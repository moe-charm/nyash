#!/bin/bash
# json_query_vm.sh — Example app: JSON query (VM)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

APP_DIR="$NYASH_ROOT/apps/examples/json_query"
# Use default dev behavior (rewrite enabled) for stable resolution
# Dev-time guard: tolerate compare on Void while VM fallback is being hardened
export NYASH_VM_TOLERATE_VOID=1
output=$(run_nyash_vm "$APP_DIR/main.nyash" --dev)

expected=$(cat << 'TXT'
2
"x"
{"b":[1,2,3]}
[1,2,3]
null
null
1
"v"
10
null
null
TXT
)

compare_outputs "$expected" "$output" "json_query_vm" || exit 1
