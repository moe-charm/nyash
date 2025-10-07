#!/bin/bash
# hakorune_vm_m3_phi_diamond_vm.sh — PHI with two incomings picks by predecessor (Hakorune)
# tags: selfhost hakorune

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/hakorune_vm_m3_phi_diamond_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF_NY'
using hakorune.vm.entry as HakoruneVmEntryBox

static box Main {
  main() {
    // bb0: const 7->1; jump 2
    // bb1: const 9->2; jump 2
    // bb2: phi dst 3 = { pred 0 -> 1, pred 1 -> 2 }; ret 3
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[" +
      "{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":7}},{\"op\":\"jump\",\"target\":2}]}," +
      "{\"id\":1,\"instructions\":[{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":9}},{\"op\":\"jump\",\"target\":2}]}," +
      "{\"id\":2,\"instructions\":[{\"op\":\"phi\",\"dst\":3,\"values\":[{\"pred\":0,\"value\":1},{\"pred\":1,\"value\":2}]},{\"op\":\"ret\",\"value\":3}]}]}]}"
    local v = HakoruneVmEntryBox.run_min(j)
    print(HakoruneVmEntryBox.int_to_str(v))
    return 0
  }
}
EOF_NY

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="7"
compare_outputs "$expected" "$out" "hakorune_vm_m3_phi_diamond_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
