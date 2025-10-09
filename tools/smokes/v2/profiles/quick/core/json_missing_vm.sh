#!/bin/bash
# json_missing_vm.sh — Verify MissingBox print observation (VM, dev-only flags)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Enable file-using and Missing observation
export NYASH_ALLOW_USING_FILE=1
export NYASH_NULL_MISSING_BOX=1

code='using "apps/lib/json_native/core/node.nyash" as JsonNode
static box Main { main() {
  local o = JsonNode.create_object()
  // Missing key access
  print(o.object_get("nope"))
  return 0
} }'

output=$(run_nyash_vm -c "$code" --dev)
if echo "$output" | grep -q 'Static box field access is not supported'; then
  log_warn "SKIP json_missing_vm (static self field in lib)"
  exit 0
fi

expected=$(cat << 'TXT'
null
TXT
)

compare_outputs "$expected" "$output" "json_missing_vm" || exit 1
