#!/bin/bash
# wasm_std_array_push_get_vm.sh — VM stubbed nykernel.* with ArrayBox push/get/size

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export NYASH_USING=1
export SMOKES_USE_DEV=1
export NYASH_ENABLE_NYKERNEL_STUB=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/wasm_std_array_push_get_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'NY'
using "apps/hakorune/std/core/array.hako" as StdArray

static box Main {
  main() {
    local env = 0
    local a = new ArrayBox()
    // auto-birth enabled; if needed: a.birth()
    a.push(7)
    a.push(11)
    print("" + a.size())
    print("" + a.get(0))
    print("" + a.get(1))
    return 0
  }
}
NY

out=$(run_nyash_vm "$SRC")
want=$(cat << 'E'
2
7
11
E
)
compare_outputs "$want" "$out" "wasm_std_array_push_get_vm" || { rm -rf "$TMP_DIR"; exit 1; }
rm -rf "$TMP_DIR"
exit 0
