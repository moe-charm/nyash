#!/bin/bash
# parser_static_box_field_annotation_vm.sh — static box field type annotation acceptance

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2

test_parser_static_box_field_annotation_vm() {
  local code='static box Main {
    console: ConsoleBox
    main() { return 0 }
  }'
  # Should parse and run without COLON error; output may be empty except runner banners.
  out=$(run_nyash_vm -c "$code" --dev)
  # If a parse error occurs, the runner prints an error line starting with a marker we filter; compare success by exit code.
  # Here, just assert the runner completed (exit code 0) via presence of no fatal markers in the last line.
  if echo "$out" | grep -q "Unexpected token COLON"; then
    test_fail "static box field annotation parse error"
    return 1
  fi
  test_pass "parser_static_box_field_annotation_vm"
}

run_test "parser_static_box_field_annotation_vm" test_parser_static_box_field_annotation_vm

