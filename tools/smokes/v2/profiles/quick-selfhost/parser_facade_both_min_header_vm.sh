#!/usr/bin/env bash
# parser_facade_both_min_header_vm.sh — Stage-4 minimal C ABI harness sanity

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_parser_facade_both_min_header_vm() {
  # Requires the harness feature to be built in; otherwise SKIP.
  if ! ./target/release/nyash --version 2>/dev/null | grep -q 'features'; then
    test_skip "nyash not built"; return 0
  fi
  # Build the helper bin name (rust produces it as standalone)
  if [ ! -x ./target/release/parser_facade_both_min ]; then
    # Try to build quickly
    if ! cargo build --release --features parser-c-abi >/dev/null 2>&1 ; then
      test_skip "parser-c-abi not available"; return 0
    fi
  fi
  local code=$'static box Main {\n  main() {\n    let a = 1; a; return 0;\n  }\n}\n'
  SMOKES_PARSER_MODE=both ./target/release/parser_facade_both_min -c "$code" >/dev/null 2>&1 || {
    test_fail "both-mode header parity failed"; return 1; }
  test_pass parser_facade_both_min_header_vm
}

run_test parser_facade_both_min_header_vm test_parser_facade_both_min_header_vm

