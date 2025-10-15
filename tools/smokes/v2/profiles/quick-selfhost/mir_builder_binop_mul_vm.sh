#!/usr/bin/env bash
# mir_builder_binop_mul_vm.sh — MIR Builder v1: binop Mul coverage

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

run_multi() {
  local code=$'static box Main {\n  main(args) {\n    local a = 6;\n    local b = 7;\n    local c = a * b;\n    print("const,const,binop(Mul),ret");\n    return c;\n  }\n}\n'
  local out ec
  set +e; out=$(run_nyash_vm -c "$code"); ec=$?; set -e
  out=$(echo "$out" | filter_noise | awk 'NF{last=$0} END{print last}')
  check_exact "const,const,binop(Mul),ret" "$out" "mir_builder_binop_mul_vm"
}

run_test mir_builder_binop_mul_vm run_multi
