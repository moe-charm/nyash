#!/bin/bash
# flow_forbid_me_vm.sh — Flow methods must not use 'me'

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_flow_forbid_me() {
  export NYASH_ENABLE_FLOW=1
  local TMP_DIR="/tmp/flow_forbid_me_vm_$$"
  mkdir -p "$TMP_DIR"
  cat > "$TMP_DIR/code.nyash" << 'EOF'
flow Main {
  main() {
    me
    return 0
  }
}
EOF
  # Expect parser/semantic error mentioning 'me' forbiddance
  if check_error_pattern "$TMP_DIR/code.nyash" "flow methods have no receiver|not allowed|\bme\b" "flow_forbid_me"; then
    rm -rf "$TMP_DIR"; return 0
  else
    rm -rf "$TMP_DIR"; return 1
  fi
}

run_test "flow_forbid_me" test_flow_forbid_me

