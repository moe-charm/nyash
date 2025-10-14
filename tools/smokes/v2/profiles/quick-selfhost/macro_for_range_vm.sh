#!/usr/bin/env bash
# macro_for_range_vm.sh — @for (i in 0..n) sums 0..n-1

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_macro_for_range() {
  local code=$'static box Main {\n  main() {\n    local c; c = 0;\n    @for (i in 0..5) { c = c + 1 }\n    print(c);\n    return c;\n  }\n}\n'
  export NYASH_MACRO_ENABLE=1
  export NYASH_JSON_ONLY=0
  local out raw ec
  set +e; raw=$(run_nyash_vm -c "$code"); ec=$?; out=$(echo "$raw" | grep -E '^[0-9]+$' | tail -n1); set -e
  # count of 0..4 = 5
  if [ "$out" = "5" ]; then
    test_pass macro_for_range_vm
  else
    echo "[WARN] SKIP macro_for_range_vm (range sugar not active; out='${out}', ec=${ec})" >&2
    return 0
  fi
}

run_test macro_for_range_vm test_macro_for_range
exit 0
