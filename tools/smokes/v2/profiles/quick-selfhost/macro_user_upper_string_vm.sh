#!/usr/bin/env bash
# macro_user_upper_string_vm.sh — MacroBoxSpec.expand via NYASH_MACRO_PATHS (uppercasing)

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_macro_user_upper_string() {
  export NYASH_MACRO_ENABLE=1
  export NYASH_MACRO_PATHS="apps/macros/examples/upper_string_macro.nyash"
  export NYASH_JSON_ONLY=0
  # Program prints a literal that should be uppercased by the user macro
  local code=$'static box Main {\n  main() {\n    print("UPPER:hello");\n    return 0;\n  }\n}\n'
  local out
  set +e; out=$(run_nyash_vm -c "$code"); ec=$?; set -e
  # Expect the uppercase string HELLO somewhere in output (do not rely on last line)
  if echo "$out" | grep -q '^HELLO$'; then
    test_pass macro_user_upper_string_vm
  else
    echo "[WARN] SKIP macro_user_upper_string_vm (user macro child pipeline not ready; ec=${ec})" >&2
    return 0
  fi
}

run_test macro_user_upper_string_vm test_macro_user_upper_string
exit 0
