#!/bin/bash
# flow_basic_vm.sh — basic flow entry main() smoke (dev-gated)

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_flow_basic_main() {
  # Enable flow syntax for this test
  export NYASH_ENABLE_FLOW=1

  local TMP_DIR="/tmp/flow_basic_vm_$$"
  mkdir -p "$TMP_DIR"
  cat > "$TMP_DIR/code.nyash" << 'EOF'
flow Main {
  main() {
    print("ok")
    return 0
  }
}
EOF

  local output
  output=$(run_nyash_vm "$TMP_DIR/code.nyash" 2>&1 || true)
  rm -rf "$TMP_DIR"
  check_exact "ok" "$output" "flow_basic_main"
}

run_test "flow_basic_main" test_flow_basic_main

