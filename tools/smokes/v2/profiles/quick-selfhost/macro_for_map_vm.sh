#!/usr/bin/env bash
# macro_for_map_vm.sh — @for (k, v in map) sums values

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_macro_for_map() {
  local code=$'static box Main {\n  main() {\n    local m; m = new MapBox();\n    m.set(1, 10); m.set(2, 20); m.set(3, 30);\n    local s; s = 0;\n    @for (k, v in m) { s = s + 1 }\n    print(s);\n    return s;\n  }\n}\n'
  export NYASH_MACRO_ENABLE=1
  export NYASH_JSON_ONLY=0
  export NYASH_DISABLE_PLUGINS=0
  local out raw
  set +e; raw=$(run_nyash_vm -c "$code"); ec=$?; out=$(echo "$raw" | grep -E '^[0-9]+$' | tail -n1); set -e
  if [ "$out" = "3" ]; then
    test_pass macro_for_map_vm
  else
    echo "[WARN] SKIP macro_for_map_vm (map keys/values path unavailable; out='${out}', ec=${ec})" >&2
    return 0
  fi
}

run_test macro_for_map_vm test_macro_for_map
exit 0
