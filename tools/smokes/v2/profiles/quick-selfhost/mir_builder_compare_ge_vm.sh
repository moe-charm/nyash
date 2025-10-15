#!/usr/bin/env bash
# mir_builder_compare_ge_vm.sh — MIR Builder compare >= coverage

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

run_compare_ge() {
  local code=$'static box Main {\n  main(args) {\n    local a = 10;\n    local b = 10;\n    local r;\n    if (a >= b) {\n      r = 1;\n    } else {\n      r = 0;\n    }\n    print("const,const,compare(Ge),branch");\n    return r;\n  }\n}\n'
  local out ec
  set +e; out=$(run_nyash_vm -c "$code"); ec=$?; set -e
  out=$(echo "$out" | filter_noise | awk 'NF{last=$0} END{print last}')
  check_exact "const,const,compare(Ge),branch" "$out" "mir_builder_compare_ge_vm"
}

run_test mir_builder_compare_ge_vm run_compare_ge
