#!/bin/bash
# json_stringify_standard_vm.sh — JSON.stringify(any) is first-class (no dev gate)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/json_stringify_standard_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
static box Main {
  main() {
    // Build nested Map/Array structure
    local m = new MapBox()
    m.set("a", 1)
    m.set("b", 2)
    local arr = new ArrayBox()
    arr.push(7)
    arr.push(3)
    m.set("list", arr)

    // JSON.stringify(any) should equal .toJSON()
    local s1 = JSON.stringify(m)
    local s2 = m.toJSON()
    if s1 == s2 { print("ok") } else { print("ng") }
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" | tail -n 1 | tr -d '\r' | xargs)
expected="ok"
compare_outputs "$expected" "$out" "json_stringify_standard_vm" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

