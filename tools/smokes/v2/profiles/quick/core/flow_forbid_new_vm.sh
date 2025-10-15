#!/bin/bash
# flow_forbid_new_vm.sh — new Flow() should be forbidden

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_flow_forbid_new() {
  export NYASH_ENABLE_FLOW=1
  # Disable using prelude entirely for this focused check (avoid unrelated parse/noise)
  export NYASH_USING=0
  export NYASH_USING_AST=1
  local TMP_DIR="/tmp/flow_forbid_new_vm_$$"
  mkdir -p "$TMP_DIR"
  cat > "$TMP_DIR/code.nyash" << 'EOF'
flow Main {
  main() {
    local x
    x = new Main()
    print(x)
    return 0
  }
}
EOF
  if check_error_pattern "$TMP_DIR/code.nyash" "Cannot instantiate static/flow|Unknown Box type|Parse error" "flow_forbid_new"; then
    rm -rf "$TMP_DIR"; return 0
  else
    rm -rf "$TMP_DIR"; return 1
  fi
}

run_test "flow_forbid_new" test_flow_forbid_new
