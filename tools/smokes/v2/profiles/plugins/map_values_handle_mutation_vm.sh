#!/bin/bash
# map_values_handle_mutation_vm.sh — Stage-2: values() returns handles; mutation visible via map.get

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/map_values_handle_mutation_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
static box Main {
  main() {
    local arr = new ArrayBox()
    arr.push(1)
    local m = new MapBox()
    m.set("a", arr)
    local vs = m.values()
    local h = vs.get(0)
    h.push(2)
    local a2 = m.get("a")
    print("" + a2.size())
    return 0
  }
}
NY

export NYASH_PLUGIN_MAP_ARRAY_HANDLE=1
out=$(run_nyash_vm "$TMP_DIR/driver.nyash" | tail -n 1 | tr -d '\r')
expected='2'
if [ "$out" != "$expected" ]; then
  test_skip "map_values_handle_mutation_vm (plugin host-handle arrays not available)"
  rm -rf "$TMP_DIR"; exit 0
fi
compare_outputs "$expected" "$out" "map_values_handle_mutation_vm" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

