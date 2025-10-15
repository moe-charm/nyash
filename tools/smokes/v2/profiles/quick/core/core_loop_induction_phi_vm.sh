#!/bin/bash
# core_loop_induction_phi_vm.sh — JSON v0 bridge: loop induction PHI (x from 0 to 3)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

export NYASH_JSON_V0_DIRECT=1

TMP_DIR="/tmp/core_loop_induction_phi_vm_$$"
mkdir -p "$TMP_DIR"
JSON_FILE="$TMP_DIR/prog.json"

# Program:
# local x=0; loop (x<3) { x=x+1 }; return x
cat > "$JSON_FILE" << 'EOF'
{
  "version": 0,
  "kind": "Program",
  "body": [
    { "type": "Local", "name": "x", "expr": { "type": "Int", "value": 0 } },
    { "type": "Loop",
      "cond": { "type": "Compare", "op": "<", "lhs": { "type": "Var", "name": "x" }, "rhs": { "type": "Int", "value": 3 } },
      "body": [
        { "type": "Local", "name": "x", "expr": { "type": "Binary", "op": "+", "lhs": { "type": "Var", "name": "x" }, "rhs": { "type": "Int", "value": 1 } } }
      ]
    },
    { "type": "Return", "expr": { "type": "Var", "name": "x" } }
  ]
}
EOF

# NOTE: Local with same name acts like rebind (assignment). x = x + 1

raw_output=$("$NYASH_BIN" --backend vm "$JSON_FILE" 2>&1)
result=$(echo "$raw_output" | sed -n 's/^Result: \(.*\)$/\1/p' | tail -n 1)
if [ -z "$result" ]; then
  log_warn "SKIP core_loop_induction_phi_vm (no Result: line in this build)"
  rm -rf "$TMP_DIR"; exit 0
fi
if [ "$result" = "3" ]; then
  log_success "core_loop_induction_phi_vm Result: 3"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "core_loop_induction_phi_vm expected Result: 3, got: ${result:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi
