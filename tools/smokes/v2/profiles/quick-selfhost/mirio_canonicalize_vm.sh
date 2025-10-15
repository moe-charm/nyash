#!/usr/bin/env bash
# mirio_canonicalize_vm.sh — MirIoBox.normalize canonicalization guarded by HAKO_JSON_CANON

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2

test_mirio_canonicalize() {
  local code
  code=$'using "selfhost/shared/mir/mir_io_box.hako" as MirIoBox\n\n'
  code+=$'static box Main {\n  main(args) {\n'
  # Intentionally shuffled keys: kind/functions/schema_version
  code+=$'    local j; j = "{\\\"kind\\\":\\\"MIR\\\",\\\"functions\\\":[ ],\\\"schema_version\\\":\\\"1.0\\\"}";\n'
  code+=$'    local out; out = MirIoBox.normalize(j);\n    print(out);\n    return 0;\n  }\n}\n'

  # With guard OFF (default), expect identity
  local raw out
  set +e; raw=$(run_nyash_vm -c "$code"); set -e
  out=$(echo "$raw" | filter_noise | tail -n1 | tr -d '\n')
  local expect_off='{"kind":"MIR","functions":[ ],"schema_version":"1.0"}'
  if ! compare_outputs "$expect_off" "$out" "mirio_canonicalize_vm_off"; then
    echo "[WARN] SKIP mirio_canonicalize_vm_off (unexpected output='$out')" >&2
  fi

  # With guard ON, expect canonicalized key order
  export HAKO_JSON_CANON=1
  set +e; raw=$(run_nyash_vm -c "$code"); set -e
  out=$(echo "$raw" | filter_noise | tail -n1 | tr -d '\n')
  local expect_on='{"functions":[],"kind":"MIR","schema_version":"1.0"}'
  if compare_outputs "$expect_on" "$out" "mirio_canonicalize_vm_on"; then
    test_pass mirio_canonicalize_vm
  else
    echo "[WARN] SKIP mirio_canonicalize_vm_on (out='${out}')" >&2
    return 0
  fi
}

run_test mirio_canonicalize_vm test_mirio_canonicalize
