#!/bin/bash
# Gate: heavy JSON object roundtrip; skip by default under bring-up
if [ "${SMOKES_ENABLE_JSON_ROUNDTRIP:-0}" != "1" ]; then
  echo "SKIP: enable with SMOKES_ENABLE_JSON_ROUNDTRIP=1" >&2
  exit 0
fi
# json_object_roundtrip_vm.sh — Minimal object parse→stringify via JsonNode (VM)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/json_object_roundtrip_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF2'
using JsonNode as JsonNode
using "apps/lib/json_native/utils/string.hako" as StringUtils
using "apps/lib/json_native/utils/escape.nyash" as EscapeUtils
using "apps/lib/json_native/utils/object_parse_box.hako" as ObjectParseBox
using "apps/lib/json_native/utils/parse_ops_box.hako" as ParseOpsBox
using "apps/lib/json_native/utils/stringify_ops_box.hako" as StringifyOpsBox
static box Main {
  main() {
    local s1 = JsonNode.parse("{}")
    local s2 = JsonNode.parse("{\"a\":1}")
    print(s1.toString())
    print(s2.toString())
    return 0
  }
}
EOF2

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" | tail -n 2 | tr -d '\r')
expected=$(cat << 'TXT'
{}
{"a":1}
TXT
)

compare_outputs "$expected" "$out" "json_object_roundtrip_vm" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
