#!/bin/bash
# flow_utils_vm.sh — use utility flow from Main (dev-gated)

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_flow_utils_call() {
  export NYASH_ENABLE_FLOW=1
  local TMP_DIR="/tmp/flow_utils_vm_$$"
  mkdir -p "$TMP_DIR"
  cat > "$TMP_DIR/code.nyash" << 'EOF'
flow MathUtils {
  add(a, b) {
    return a + b
  }
}

flow Main {
  main() {
    local v
    v = MathUtils.add(2, 3)
    print(v)
    return 0
  }
}
EOF

  local output
  output=$(run_nyash_vm "$TMP_DIR/code.nyash" 2>&1 | grep -v '^Result: ' || true)
  rm -rf "$TMP_DIR"
  check_exact "5" "$output" "flow_utils_call"
}

run_test "flow_utils_call" test_flow_utils_call
