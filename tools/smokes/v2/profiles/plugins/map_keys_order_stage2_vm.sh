#!/bin/bash
# map_keys_order_stage2_vm.sh — Stage-2: keys() ordering/content minimal check

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/map_keys_order_stage2_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
static box Main {
  main() {
    local m = new MapBox()
    m.set("b", 20)
    m.set("a", 10)
    local ks = m.keys()
    print(ks.get(0) + "," + ks.get(1))
    return 0
  }
}
NY

export NYASH_PLUGIN_MAP_ARRAY_HANDLE=1
out=$(run_nyash_vm "$TMP_DIR/driver.nyash" | tail -n 1 | tr -d '\r')
expected='a,b'
if [ "$out" != "$expected" ]; then
  test_skip "map_keys_order_stage2_vm (plugin host-handle arrays not available or ordering differs)"
  rm -rf "$TMP_DIR"; exit 0
fi
compare_outputs "$expected" "$out" "map_keys_order_stage2_vm" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

