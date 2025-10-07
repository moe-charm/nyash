#!/bin/bash
# hakorune_vm_m3_jump_vm.sh — jump(target) transfers control to target block
# tags: selfhost hakorune

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/hakorune_vm_m3_jump_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF_NY'
using hakorune.vm.entry as HakoruneVmEntryBox

static box Main {
  main() {
    // blocks: 0 -> jump 1; 1: const r1=9; ret r1
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":["
    j = j + "{\"id\":0,\"instructions\":[{\"op\":\"jump\",\"target\":1}]},"
    j = j + "{\"id\":1,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":9}},{\"op\":\"ret\",\"value\":1}]}]}]}"
    local v = HakoruneVmEntryBox.run_min(j)
    print(HakoruneVmEntryBox.int_to_str(v))
    return 0
  }
}
EOF_NY

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="9"
compare_outputs "$expected" "$out" "hakorune_vm_m3_jump_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
