#!/bin/bash
# map_keys_values_fallback_vm.sh — Verify keysS()/valuesS() fallback path without host-handle arrays

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

if [ "${HAKO_MAP_KEYS_VALUES_FALLBACK:-1}" = "0" ]; then
  test_skip "map_keys_values_fallback_vm" "fallback disabled (HAKO_MAP_KEYS_VALUES_FALLBACK=0)"; exit 0
fi

TMP_DIR="/tmp/map_keys_values_fallback_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
static box Main {
  main() {
    local m = new MapBox()
    m.set("b", 20)
    m.set("a", 10)
    // Fallback API existence check (keysS/valuesS should return a String)
    local ks = m.keysS()
    local vs = m.valuesS()
    if ks == null || vs == null || ks.size() == 0 || vs.size() == 0 {
      print("0:0")
      return 0
    }
    // Compute sum via canonical get (alignment check is covered by stage2 test)
    local sum = 0
    sum = (0 + m.get("a")) + (0 + m.get("b"))
    print("2:" + ("" + sum))
    return 0
  }
}
NY

unset NYASH_PLUGIN_MAP_ARRAY_HANDLE
out=$(run_nyash_vm "$TMP_DIR/driver.nyash" | tail -n 1 | tr -d '
')
expected='2:30'
compare_outputs "$expected" "$out" "map_keys_values_fallback_vm" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
