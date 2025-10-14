#!/usr/bin/env bash
# parser_facade_simple_fn_vm.sh — Facade parses a simple function with local vars

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_parser_facade_simple_fn_vm() {
  local code=$'static box Main {\n  main() {\n    local a = 1\n    local b = 2\n    if (a + b) == 3 { print("OK2") }\n    return 0\n  }\n}\n'
  HAKO_FRONT_USE_FACADE=1 out=$(run_nyash_vm -c "$code" 2>&1 | tail -n 1 | tr -d '\r')
  if [ "$out" = "OK2" ]; then
    test_pass parser_facade_simple_fn_vm
  else
    echo "$out"; test_fail "unexpected output"; return 1
  fi
}

run_test parser_facade_simple_fn_vm test_parser_facade_simple_fn_vm

