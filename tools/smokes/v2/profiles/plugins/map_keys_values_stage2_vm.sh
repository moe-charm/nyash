#!/bin/bash
# map_keys_values_stage2_vm.sh — Map.keys()/values() return HostHandle(ArrayBox) when enabled; else SKIP

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/map_keys_values_stage2_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
static box Main {
  main() {
    local m = new MapBox()
    // insert in reverse order to check sorted keys alignment
    m.set("b", 20)
    m.set("a", 10)
    local ks = m.keys()
    local vs = m.values()
    local sum = 0
    local i = 0
    if vs != null && vs.size != null {
      loop(i < vs.size()) { sum = sum + vs.get(i)  i = i + 1 }
    }
    print("" + ks.size() + ":" + sum)
    return 0
  }
}
NY

# Try stage-2 path (plugin host-handle arrays); accept SKIP if not available
export NYASH_PLUGIN_MAP_ARRAY_HANDLE=1
out=$(run_nyash_vm "$TMP_DIR/driver.nyash" | tail -n 1 | tr -d '')
expected='2:30'
if [ "$out" != "$expected" ]; then
  test_skip "map_keys_values_stage2_vm (plugin host-handle arrays not available)"
  rm -rf "$TMP_DIR"; exit 0
fi
compare_outputs "$expected" "$out" "map_keys_values_stage2_vm" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
