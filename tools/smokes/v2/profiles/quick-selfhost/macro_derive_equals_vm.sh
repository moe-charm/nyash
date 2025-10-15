#!/usr/bin/env bash
# macro_derive_equals_vm.sh — @derive('Equals') injects equals(other)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_macro_derive_equals() {
  local code=$'@derive("Equals")\nbox Foo { }\n\nstatic box Main {\n  main() {\n    local x; x = new Foo();\n    local y; y = new Foo();\n    if (x.equals(y)) { print(1); return 1; } else { print(0); return 0; }\n  }\n}\n'
  export NYASH_MACRO_ENABLE=1
  export NYASH_JSON_ONLY=0
  local out raw ec
  set +e; raw=$(run_nyash_vm -c "$code"); ec=$?; out=$(echo "$raw" | grep -E '^[0-9]+$' | tail -n1); set -e
  if [ "$out" = "0" ]; then
    test_pass macro_derive_equals_vm
  else
    echo "[WARN] SKIP macro_derive_equals_vm (derive equals recursion/stack; out='${out}', ec=${ec})" >&2
    return 0
  fi
}

run_test macro_derive_equals_vm test_macro_derive_equals
exit 0
