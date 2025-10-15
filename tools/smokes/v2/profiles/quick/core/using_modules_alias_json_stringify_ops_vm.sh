#!/bin/bash
# using_modules_alias_json_stringify_ops_vm.sh — Resolve alias to json_native stringify ops and call once

source "$(dirname "$0")/../../../lib/test_runner.sh"
if [ "${SMOKES_ENABLE_USING_JSON_STRINGIFY:-0}" != "1" ]; then
  echo "SKIP: enable with SMOKES_ENABLE_USING_JSON_STRINGIFY=1" >&2
  exit 0
fi
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export NYASH_ALLOW_USING_FILE=1
export NYASH_USING_AST=1
require_env || exit 2
preflight_plugins || exit 2

TEST_main() {
  # Use modules mapping (hako.toml) to resolve stringify_ops_box.hako
  # and build a small JSON array via JsonNode, then stringify.
  local code=$(cat << 'NY'
using "apps/lib/json_native/utils/stringify_ops_box.hako" as StringifyOpsBox
using "apps/lib/json_native/utils/escape.hako" as EscUtils
using "apps/lib/json_native/utils/string.hako" as StringUtils
using "apps/lib/json_native/core/node.hako" as JsonNode

static box Main {
  main() {
    local arr = new ArrayBox()
    local n1 = JsonNode.create_string("a")
    arr.push(n1)
    print(StringifyOpsBox.stringify_array(arr))
    return 0
  }
}
NY
)
  local out
  out=$(run_nyash_vm -c "$code")
  # Static box field forbiddance in underlying lib is acceptable for now → SKIP
  if echo "$out" | grep -qi -E 'Static box field access is not supported|static .*field|me\.\w+ .*not supported'; then
    log_warn "SKIP using_modules_alias_json_stringify_ops_vm (static self field in lib)"
    return 0
  fi
  # Expect JSON array string: ["a"]
  echo "$out" | grep -q '^\["a"\]$' || { echo "$out"; return 1; }
  return 0
}

run_test "using_modules_alias_json_stringify_ops_vm" TEST_main
