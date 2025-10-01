#!/bin/bash
# flow_forbid_birth_fini_vm.sh — Flow must not declare birth/fini

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_flow_forbid_birth() {
  export NYASH_ENABLE_FLOW=1
  local TMP_DIR="/tmp/flow_forbid_birth_vm_$$"
  mkdir -p "$TMP_DIR"
  cat > "$TMP_DIR/code.nyash" << 'EOF'
flow Main {
  birth() { return 0 }
}
EOF
  if check_error_pattern "$TMP_DIR/code.nyash" "flow forbids birth/fini|birth" "flow_forbid_birth"; then
    rm -rf "$TMP_DIR"; return 0
  else
    rm -rf "$TMP_DIR"; return 1
  fi
}

test_flow_forbid_fini() {
  export NYASH_ENABLE_FLOW=1
  local TMP_DIR="/tmp/flow_forbid_fini_vm_$$"
  mkdir -p "$TMP_DIR"
  cat > "$TMP_DIR/code.nyash" << 'EOF'
flow Main {
  fini() { return 0 }
}
EOF
  if check_error_pattern "$TMP_DIR/code.nyash" "flow forbids birth/fini|fini" "flow_forbid_fini"; then
    rm -rf "$TMP_DIR"; return 0
  else
    rm -rf "$TMP_DIR"; return 1
  fi
}

run_test "flow_forbid_birth" test_flow_forbid_birth
run_test "flow_forbid_fini" test_flow_forbid_fini

