#!/bin/bash
# parser_static_box_field_init_vm.sh — Accept static box field annotations with initializer (declarative only)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2

test_parser_static_box_field_init() {
  local TMP_DIR="/tmp/parser_static_box_field_init_$$"
  mkdir -p "$TMP_DIR"
  cat > "$TMP_DIR/code.nyash" << 'EOF'
static box Config {
  debug: IntegerBox = 1   // Declarative: accepted by parser; no runtime state
}
static box Main { main(){
  // Do not touch me.field; only ensure parse/build succeeds
  print("ok")
  return 0
}}
EOF
  local output
  if output=$(run_nyash_vm "$TMP_DIR/code.nyash" 2>&1 | grep -v '^Result: ' | filter_noise); then
    check_exact "ok" "$output" "parser_static_box_field_init"
  else
    echo "$output" >&2
    return 1
  fi
}

run_test "parser_static_box_field_init" test_parser_static_box_field_init
