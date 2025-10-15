#!/bin/bash
# Gate: enable when result line capture is desired (filtered by default)
if [ "${SMOKES_ENABLE_JSON_V0_RESULT:-0}" != "1" ]; then
  echo "SKIP: enable with SMOKES_ENABLE_JSON_V0_RESULT=1" >&2
  exit 0
fi
# json_v0_match_phi_vm.sh — JSON v0 bridge: match basic (two arms + else)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_USE_DEV=1
require_env || exit 2
preflight_plugins || exit 2

# Direct JSON v0 parse
export NYASH_JSON_V0_DIRECT=1

TMP_DIR="/tmp/json_v0_match_phi_vm_$$"
mkdir -p "$TMP_DIR"
JSON_FILE="$TMP_DIR/prog.json"

cat > "$JSON_FILE" << 'EOF'
{
  "version": 0,
  "kind": "Program",
  "body": [
    { "type": "Return",
      "expr": { "type": "Match",
        "scrutinee": { "type": "Str", "value": "a" },
        "arms": [
          { "label": "a", "expr": { "type": "Int", "value": 7 } },
          { "label": "b", "expr": { "type": "Int", "value": 3 } }
        ],
        "else": { "type": "Int", "value": 1 }
      }
    }
  ]
}
EOF

raw_output=$(run_nyash_vm "$JSON_FILE")
result=$(echo "$raw_output" | sed -n 's/^Result: \(.*\)$/\1/p' | tail -n 1)
if [ "$result" = "7" ]; then
  log_success "json_v0_match_phi_vm Result: 7"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "json_v0_match_phi_vm expected Result: 7, got: ${result:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi
