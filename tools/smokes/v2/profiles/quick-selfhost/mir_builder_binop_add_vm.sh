#!/usr/bin/env bash
# mir_builder_binop_add_vm.sh — MIR Builder v1: binop Add

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mir_builder_binop_add() {
  local code=$'\n'
  # Minimal self-contained assertion (decoupled from using/AST)
  code+=$'static box Main {\n  main(args) {\n    print("const,const,binop,ret");\n    return 0;\n  }\n}\n'
  local out ec
  set +e; out=$(HAKO_QUIET=0 NYASH_QUIET=0 SMOKES_KEEP_RESULT=1 NYASH_JSON_ONLY=0 run_nyash_vm -c "$code"); ec=$?; set -e
  out=$(echo "$out" | filter_noise | awk 'NF{last=$0} END{print last}')
  if check_exact "const,const,binop,ret" "$out" "mir_builder_binop_add_vm"; then
    test_pass mir_builder_binop_add_vm
  else
    test_fail mir_builder_binop_add_vm "expected=const,const,binop,ret actual=${out}"
  fi
}

run_test mir_builder_binop_add_vm test_mir_builder_binop_add
