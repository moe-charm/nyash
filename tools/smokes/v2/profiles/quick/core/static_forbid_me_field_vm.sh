#!/bin/bash
# static_forbid_me_field_vm.sh — Static box forbids `me.field` access/assignment

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2

test_static_forbid_me_field() {
  local TMP_DIR="/tmp/static_forbid_me_field_$$"
  mkdir -p "$TMP_DIR"
  cat > "$TMP_DIR/code.nyash" << 'EOF'
static box Config {
  debug: IntegerBox
  set() { me.debug = 1 }
}
static box Main {
  main() {
    Config.set()
    return 0
  }
}
EOF
  if check_error_pattern "$TMP_DIR/code.nyash" "Static box field (access|assignment) is not supported|Static box field" "static_forbid_me_field"; then
    return 0
  else
    return 1
  fi
}

run_test "static_forbid_me_field" test_static_forbid_me_field
