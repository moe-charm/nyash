#!/bin/bash
# Gate: run only when explicitly enabled (VM PHI unify is environment-dependent)
if [ "${SMOKES_ENABLE_JSON_V0_UNIFY:-0}" != "1" ]; then
  echo "SKIP: enable with SMOKES_ENABLE_JSON_V0_UNIFY=1" >&2
  exit 0
fi
export NYASH_JSONV0_PHI_UNIFY=1
# json_v0_if_return_phi_vm.sh — JSON v0 bridge: If with then=Return, else=Local; merge must take else value

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

# Direct JSON v0 parse
export NYASH_JSON_V0_DIRECT=1

TMP_DIR="/tmp/json_v0_if_return_phi_vm_$$"
mkdir -p "$TMP_DIR"
JSON_FILE="$TMP_DIR/prog.json"

cat > "$JSON_FILE" << 'EOF'
{
  "version": 0,
  "kind": "Program",
  "body": [
    { "type": "If",
      "cond": { "type": "Bool", "value": false },
      "then": [
        { "type": "Local", "name": "x", "expr": { "type": "Int", "value": 7 } },
        { "type": "Return", "expr": { "type": "Int", "value": 0 } }
      ],
      "else": [
        { "type": "Local", "name": "x", "expr": { "type": "Int", "value": 5 } }
      ]
    },
    { "type": "Return", "expr": { "type": "Var", "name": "x" } }
  ]
}
EOF

raw_output=$(run_nyash_vm "$JSON_FILE")
if [ "${SMOKES_DEV_LOG:-0}" = "1" ]; then echo "$raw_output" >&2; fi
result=$(echo "$raw_output" | sed -n 's/^Result: \(.*\)$/\1/p' | tail -n 1)
if [ "$result" = "5" ]; then
  log_success "json_v0_if_return_phi_vm Result: 5 (then unreachable pruned)"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "json_v0_if_return_phi_vm expected Result: 5, got: ${result:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi

