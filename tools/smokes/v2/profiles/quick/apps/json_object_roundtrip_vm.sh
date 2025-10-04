#!/bin/bash
# json_object_roundtrip_vm.sh — JsonNode.parse → stringify roundtrip (object)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

test_json_object_roundtrip_vm() {
  local code='
using "apps/lib/json_native/core/node.hako" as JsonNode
static box Main {
  main() {
  local j = "{\"a\":1,\"b\":\"x\\\"y\",\"c\":true}"
  local n = JsonNode.parse(j)
  print(n.stringify())
  return 0
  }
'
  local out
  out=$(run_nyash_vm -c "$code" --dev | awk 'match($0,/^\{.*\}$/){print; exit}')
  local expect='{"a":1,"b":"x\"y","c":true}'
  compare_outputs "$expect" "$out" "json_object_roundtrip_vm" || return 1
  return 0
}

run_test "json_object_roundtrip_vm" test_json_object_roundtrip_vm || exit 1
exit 0
