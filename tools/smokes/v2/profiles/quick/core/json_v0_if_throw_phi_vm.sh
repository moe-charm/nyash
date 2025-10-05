#!/bin/bash
# json_v0_if_throw_phi_vm.sh — JSON v0 bridge: if-then Throw should not contribute PHI input

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

# Disabled by default because VM interpreter may not fully support Throw terminator here.
if [ "${SMOKES_ENABLE_JSON_V0_THROW:-0}" != "1" ]; then
  test_skip "json_v0_if_throw_phi_vm (requires Throw support)" \
    "Enable with SMOKES_ENABLE_JSON_V0_THROW=1"
  exit 0
fi

# Enable direct JSON v0 bridge (raw JSON) and throw lowering
export NYASH_JSON_V0_DIRECT=1
export NYASH_BRIDGE_THROW_ENABLE=1

TMP_DIR="/tmp/json_v0_if_throw_phi_vm_$$"
mkdir -p "$TMP_DIR"
JSON_FILE="$TMP_DIR/prog.json"

# Program: let x; if true { throw 42 } else { x = 7 }; return x
# Expect: PHI only from else leg → result 7
cat > "$JSON_FILE" << 'JSON'
{
  "version": 0,
  "kind": "Program",
  "body": [
    { "type": "Let", "name": "x", "expr": { "type": "Int", "value": 0 } },
    { "type": "If", "cond": { "type": "Bool", "value": true },
      "then": { "type": "Throw", "expr": { "type": "Int", "value": 42 } },
      "else": { "type": "Assign", "name": "x", "expr": { "type": "Int", "value": 7 } }
    },
    { "type": "Return", "expr": { "type": "Name", "name": "x" } }
  ]
}
JSON

raw_output=$(run_nyash_vm "$JSON_FILE")
if [ "${SMOKES_DEV_LOG:-0}" = "1" ]; then echo "$raw_output" >&2; fi

result=$(echo "$raw_output" | sed -n 's/^Result: \(.*\)$/\1/p' | tail -n 1)
if [ "$result" = "7" ]; then
  log_success "json_v0_if_throw_phi_vm Result: 7 (unreachable then leg pruned)"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "json_v0_if_throw_phi_vm expected Result: 7, got: ${result:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi
