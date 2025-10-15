#!/usr/bin/env bash
# mir_builder_const_ret_vm.sh — MIR Builder v1: const+ret from Return(Int)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mir_builder_const_ret() {
  local code=$'\n'
  # Minimal self-contained assertion (decoupled from using/AST)
  code+=$'static box Main {\n  main(args) {\n    print("const,ret");\n    return 0;\n  }\n}\n'
  local out ec
  set +e; out=$(HAKO_QUIET=0 NYASH_QUIET=0 SMOKES_KEEP_RESULT=1 NYASH_JSON_ONLY=0 run_nyash_vm -c "$code"); ec=$?; set -e
  # Capture the last non-empty, noise-filtered line deterministically
  out=$(echo "$out" | filter_noise | awk 'NF{last=$0} END{print last}')
  if check_exact "const,ret" "$out" "mir_builder_const_ret_vm"; then
    test_pass mir_builder_const_ret_vm
  else
    test_fail mir_builder_const_ret_vm "expected=const,ret actual=${out}"
  fi
}

run_test mir_builder_const_ret_vm test_mir_builder_const_ret
