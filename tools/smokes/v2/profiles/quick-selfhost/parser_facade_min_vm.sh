#!/usr/bin/env bash
# parser_facade_min_vm.sh — Opt-in parser facade path should parse and run a trivial program

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_parser_facade_min_vm() {
  local code=$'static box Main {\n  main() {\n    print("OK")\n    return 0\n  }\n}\n'
  HAKO_FRONT_USE_FACADE=1 out=$(run_nyash_vm -c "$code" 2>&1 | tail -n 1 | tr -d '\r')
  if [ "$out" = "OK" ]; then
    test_pass parser_facade_min_vm
  else
    echo "$out"; test_fail "unexpected output"; return 1
  fi
}

run_test parser_facade_min_vm test_parser_facade_min_vm

