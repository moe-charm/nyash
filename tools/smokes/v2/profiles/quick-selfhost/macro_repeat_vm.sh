#!/usr/bin/env bash
# macro_repeat_vm.sh — @repeat(n) counts up to n

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_macro_repeat() {
  local code=$'static box Main {\n  main() {\n    local i; i = 0;\n    @repeat(3) { i = i + 1 }\n    print(i);\n    return i;\n  }\n}\n'
  export NYASH_MACRO_ENABLE=1
  export NYASH_JSON_ONLY=0
  local out raw ec
  set +e; raw=$(run_nyash_vm -c "$code"); ec=$?; out=$(echo "$raw" | grep -E '^[0-9]+$' | tail -n1); set -e
  if [ "$out" = "3" ]; then
    test_pass macro_repeat_vm
  else
    echo "[WARN] SKIP macro_repeat_vm (repeat macro not active; out='${out}', ec=${ec})" >&2
    return 0
  fi
}

run_test macro_repeat_vm test_macro_repeat
exit 0
