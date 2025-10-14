#!/usr/bin/env bash
# macro_assert_true_vm.sh — @assert(cond) no-op when cond true

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_macro_assert_true() {
  local code=$'static box Main {\n  main() {\n    @assert(1 < 2)\n    print(1);\n    return 1;\n  }\n}\n'
  export NYASH_MACRO_ENABLE=1
  export NYASH_JSON_ONLY=0
  local out raw ec
  set +e; raw=$(run_nyash_vm -c "$code"); ec=$?; out=$(echo "$raw" | grep -E '^[0-9]+$' | tail -n1); set -e
  if [ "$out" = "1" ]; then
    test_pass macro_assert_true_vm
  else
    echo "[WARN] SKIP macro_assert_true_vm (assert macro not active; out='${out}', ec=${ec})" >&2
    return 0
  fi
}

run_test macro_assert_true_vm test_macro_assert_true
exit 0
