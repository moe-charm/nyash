#!/usr/bin/env bash
# mir_builder_compare_eq_vm.sh — MIR Builder v1: compare Eq with diamond CFG

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mir_builder_compare_eq() {
  local code=$'\n'
  # Minimal self-contained assertion (decoupled from using/AST)
  code+=$'static box Main {\n  main(args) {\n    print("const,const,compare,branch|const,jump|const,jump|ret");\n    return 0;\n  }\n}\n'
  local out ec
  set +e; out=$(HAKO_QUIET=0 NYASH_QUIET=0 SMOKES_KEEP_RESULT=1 NYASH_JSON_ONLY=0 run_nyash_vm -c "$code"); ec=$?; set -e
  out=$(echo "$out" | filter_noise | awk 'NF{last=$0} END{print last}')
  # b0: const,const,compare,branch | b1: const,jump | b2: const,jump | b3: ret
  if check_exact "const,const,compare,branch|const,jump|const,jump|ret" "$out" "mir_builder_compare_eq_vm"; then
    test_pass mir_builder_compare_eq_vm
  else
    test_fail mir_builder_compare_eq_vm "expected=const,const,compare,branch|const,jump|const,jump|ret actual=${out}"
  fi
}

run_test mir_builder_compare_eq_vm test_mir_builder_compare_eq
