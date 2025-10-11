#!/bin/bash
# op_eq_box_identity_vm.sh — Verify op_eq equality for Box identity (plugins)

DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
. "${DIR}/../../lib/test_runner.sh"

require_env || exit 2
export SMOKES_DISABLE_PLUGIN_CHECKS=0
export NYASH_DISABLE_PLUGINS=0
preflight_plugins || exit 2

code=$'static box Main {\n  main() {\n    local arr = new ArrayBox()\n    arr.push(1)\n    local m = new MapBox()\n    m.set("k", arr)\n    local v = m.get("k")\n    // same identity should be true\n    if not (arr == v) { return 10 }\n    // different instance should be false\n    local arr2 = new ArrayBox()\n    if arr == arr2 { return 11 }\n    return 0\n  }\n}'

if run_nyash_vm -c "$code" >/dev/null; then
  test_pass "op_eq_box_identity_vm"
  exit 0
else
  test_fail "op_eq_box_identity_vm" "non-zero rc"
  exit 1
fi
