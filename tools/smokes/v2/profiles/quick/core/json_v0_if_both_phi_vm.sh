#!/bin/bash
# json_v0_if_both_phi_vm.sh — JSON v0 bridge: Diamond basic (both incoming)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

# Direct JSON v0 parse
export NYASH_JSON_V0_DIRECT=1

TMP_DIR="/tmp/json_v0_if_both_phi_vm_$$"
mkdir -p "$TMP_DIR"
JSON_FILE="$TMP_DIR/prog.json"

# Program:
# if (true) { local x=1 } else { local x=2 }; return x
cat > "$JSON_FILE" << 'EOF'
{
  "version": 0,
  "kind": "Program",
  "body": [
    { "type": "If",
      "cond": { "type": "Bool", "value": true },
      "then": [ { "type": "Local", "name": "x", "expr": { "type": "Int", "value": 1 } } ],
      "else": [ { "type": "Local", "name": "x", "expr": { "type": "Int", "value": 2 } } ]
    },
    { "type": "Return", "expr": { "type": "Var", "name": "x" } }
  ]
}
EOF

raw_output=$(run_nyash_vm "$JSON_FILE")
result=$(echo "$raw_output" | sed -n 's/^Result: \(.*\)$/\1/p' | tail -n 1)
if [ "$result" = "1" ]; then
  log_success "json_v0_if_both_phi_vm Result: 1"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "json_v0_if_both_phi_vm expected Result: 1, got: ${result:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi

