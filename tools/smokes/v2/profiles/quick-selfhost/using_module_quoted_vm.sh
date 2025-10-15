#!/usr/bin/env bash
# using_module_quoted_vm.sh — Quoted module using should resolve via [modules]/workspace

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_using_module_quoted() {
  local code=$'\n'
  code+=$'using "selfhost.shared.mir.builder" as BlockBuilderBox;\n'
  code+=$'static box Main {\n  main(args) {\n    print(BlockBuilderBox.const_ret_ops(7));\n    return 0;\n  }\n}\n'
  local out ec
  # Force program execution (not JSON-only) to capture print output
  set +e; out=$(NYASH_JSON_ONLY=0 run_nyash_vm -c "$code"); ec=$?; set -e
  out=$(echo "$out" | filter_noise | tail -n1)
  if check_exact "const,ret" "$out" "using_module_quoted_vm"; then
    test_pass using_module_quoted_vm
  else
    test_fail using_module_quoted_vm "out='${out}' ec=${ec}"
  fi
}

run_test using_module_quoted_vm test_using_module_quoted

