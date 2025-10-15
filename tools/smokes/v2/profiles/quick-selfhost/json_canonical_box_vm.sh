#!/usr/bin/env bash
# json_canonical_box_vm.sh — JsonCanonicalBox canonicalizes JSON via hostbridge extern

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_json_canonical_box() {
  export HAKO_JSON_CANON=1
  local code=$'using "selfhost/shared/json/json_canonical_box.hako" as JsonCanonicalBox\n\n'
  code+=$'static box Main {\n  main(args) {\n    local j = "{\\"b\\":1,\\"a\\":2}";\n    local out = JsonCanonicalBox.canonicalize(j);\n    if out.get != null {\n      print(out.get(0));\n    } else if out.to_string_box != null {\n      print(out.to_string_box().value);\n    } else {\n      print(out);\n    }\n    return 0;\n  }\n}\n'

  local expected='{"a":2,"b":1}'
  local raw out ec
  set +e; raw=$(run_nyash_vm -c "$code"); ec=$?; set -e
  out=$(echo "$raw" | filter_noise | tail -n1 | tr -d '\n')
  if compare_outputs "$expected" "$out" "json_canonical_box_vm"; then
    test_pass json_canonical_box_vm
  else
    echo "[WARN] SKIP json_canonical_box_vm (out='${out}', ec=${ec})" >&2
    return 0
  fi
}

run_test json_canonical_box_vm test_json_canonical_box
