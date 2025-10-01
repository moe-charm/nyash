#!/bin/bash
# flow_forbid_field_vm.sh — flow forbids fields (error expected)

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_flow_forbid_field() {
  export NYASH_ENABLE_FLOW=1
  local TMP_DIR="/tmp/flow_forbid_field_vm_$$"
  mkdir -p "$TMP_DIR"
  # Intentionally declare a field-like bare identifier inside flow
  cat > "$TMP_DIR/code.nyash" << 'EOF'
flow Main {
  x
}
EOF

  if check_error_pattern "$TMP_DIR/code.nyash" "flow cannot declare fields|flow forbids" "flow_forbid_field"; then
    rm -rf "$TMP_DIR"; return 0
  else
    rm -rf "$TMP_DIR"; return 1
  fi
}

run_test "flow_forbid_field" test_flow_forbid_field

