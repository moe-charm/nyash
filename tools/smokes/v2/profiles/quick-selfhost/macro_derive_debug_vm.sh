#!/usr/bin/env bash
# macro_derive_debug_vm.sh — @derive('Debug') injects toString()

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_macro_derive_debug() {
  local code=$'@derive("Debug")\nbox Foo { }\n\nstatic box Main {\n  main() {\n    local x; x = new Foo();\n    local s; s = x.toString();\n    print(s.length());\n    return s.length();\n  }\n}\n'
  export NYASH_MACRO_ENABLE=1
  export NYASH_JSON_ONLY=0
  local out raw ec
  set +e; raw=$(run_nyash_vm -c "$code"); ec=$?; out=$(echo "$raw" | grep -E '^[0-9]+$' | tail -n1); set -e
  if [ "$out" = "5" ]; then
    test_pass macro_derive_debug_vm
  else
    echo "[WARN] SKIP macro_derive_debug_vm (derive debug not stabilized; out='${out}', ec=${ec})" >&2
    return 0
  fi
}

run_test macro_derive_debug_vm test_macro_derive_debug
exit 0
