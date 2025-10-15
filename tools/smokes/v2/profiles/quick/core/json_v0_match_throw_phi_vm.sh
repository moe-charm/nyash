#!/bin/bash
# json_v0_match_throw_phi_vm.sh — JSON v0 bridge: match arm with Throw should not contribute PHI input

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

# Enable direct JSON v0 bridge (raw JSON) and throw lowering
export NYASH_JSON_V0_DIRECT=1
export NYASH_BRIDGE_THROW_ENABLE=1

TMP_DIR="/tmp/json_v0_match_throw_phi_vm_$$"
mkdir -p "$TMP_DIR"
JSON_FILE="$TMP_DIR/prog.json"

# Program: return match("C") { "A" => throw 123, "B" => 5 } else 7
# Expectation: then arm ends with Throw (unreachable to merge). With scrutinee "C", else is chosen → result 7
cat > "$JSON_FILE" << 'EOF'
{
  "version": 0,
  "kind": "Program",
  "body": [
    { "type": "Return", "expr": {
      "type": "Match",
      "scrutinee": { "type": "Str", "value": "C" },
      "arms": [
        { "label": "A", "expr": { "type": "Throw", "expr": { "type": "Int", "value": 123 } } },
        { "label": "B", "expr": { "type": "Int", "value": 5 } }
      ],
      "else": { "type": "Int", "value": 7 }
    }}
  ]
}
EOF

# Run directly to preserve 'Result:' line (smoke filter strips it)
raw_output=$(NYASH_VM_USE_PY=0 NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 "$NYASH_BIN" --backend vm "$JSON_FILE" 2>&1)
if [ "${SMOKES_DEV_LOG:-0}" = "1" ]; then echo "$raw_output" >&2; fi

# Runner prints MIR result lines
result=$(echo "$raw_output" | sed -n 's/^Result: \(.*\)$/\1/p' | tail -n 1)
if [ "$result" = "7" ]; then
  log_success "json_v0_match_throw_phi_vm Result: 7 (PHI pruned unreachable arm)"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "json_v0_match_throw_phi_vm expected Result: 7, got: ${result:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi
