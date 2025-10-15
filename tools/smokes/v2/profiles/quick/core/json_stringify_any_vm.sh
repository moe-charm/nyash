#!/bin/bash
# json_stringify_any_vm.sh — JSON.stringify(any) dev bridge smoke (Map/Array mixed)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Experimental guard: run only when explicitly enabled
if [[ "${NYASH_JSON_STRINGIFY_DEV:-}" != "1" ]]; then
  test_skip "JSON.stringify(any) dev bridge is experimental; set NYASH_JSON_STRINGIFY_DEV=1 to enable"
  exit 0
fi

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/json_stringify_any_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "apps/lib/json_native/stringify.hako" as JSON

static box Main {
  main() {
    // Build a nested Map/Array structure declaratively
    local mir = {
      module: "main",
      functions: [{ name: "main", blocks: [{ id: 0, instructions: [
        { op: "const", dst: 1, value: { type: "i64", value: 7 }},
        { op: "const", dst: 2, value: { type: "i64", value: 3 }},
        { op: "binop", kind: "Add", lhs: 1, rhs: 2, dst: 3 },
        { op: "ret", value: 3 }
      ]}]}]
    }
    // Compare dev bridge vs Map/Array native toJSON path
    local s1 = JSON.stringify_map(mir)
    local s2 = JSON.stringify(mir)
    if s1 == s2 { print("ok") } else { print("ng") }
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="ok"
compare_outputs "$expected" "$out" "json_stringify_any_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

