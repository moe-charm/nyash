#!/bin/bash
# map_array_handle_identity_vm.sh — Map.set(ArrayBox handle) then get() preserves identity; mutations visible

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/map_array_handle_identity_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
static box Main {
  main() {
    local arr = new ArrayBox()
    arr.push(10)
    local m = new MapBox()
    m.set("a", arr)
    local arr2 = m.get("a")
    // mutate original
    arr.push(20)
    // identity must hold → arr2.size() == 2
    print("" + arr2.size())
    return 0
  }
}
NY

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" | tail -n 1 | tr -d '')
expected='2'
compare_outputs "$expected" "$out" "map_array_handle_identity_vm" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
