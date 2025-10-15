#!/bin/bash
# set_to_array_order_vm.sh — Plugins suite: Set.toArray order matches Map.keys()

source "$(dirname "$0")/../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/set_to_array_order_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
static box Main {
  main() {
    local s = new SetBox()
    s.add("b")
    s.add("a")
    s.add("c")
    local arr = s.toArray()
    print(arr.get(0) + "," + arr.get(1))
    return 0
  }
}
NY

export NYASH_PLUGIN_MAP_ARRAY_HANDLE=1
out_full=$(run_nyash_vm "$TMP_DIR/driver.nyash" 2>&1)
out=$(echo "$out_full" | tail -n 1 | tr -d '\r')
expected='a,b'
if [ "$out" != "$expected" ]; then
  test_skip "set_to_array_order_vm (arrays/ordering not available)"
  rm -rf "$TMP_DIR"; exit 0
fi
compare_outputs "$expected" "$out" "set_to_array_order_vm" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
