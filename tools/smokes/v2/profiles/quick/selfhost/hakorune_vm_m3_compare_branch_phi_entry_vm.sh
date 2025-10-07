#!/bin/bash
# hakorune_vm_m3_compare_branch_phi_entry_vm.sh — Compare→Branch→phi(entry) composite (Hakorune)
# tags: selfhost hakorune

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/hakorune_vm_m3_compare_branch_phi_entry_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF_NY'
using hakorune.vm.entry as HakoruneVmEntryBox

static box Main {
  main() {
    // bb0: const 7->1; const 7->2; compare Eq(1,2) -> dst 3; branch cond=3 then:1 else:2
    // bb1: phi dst 4 = { pred 0 -> 1 }; ret 4
    // bb2: ret 0 (dead path)
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[" +
      "{\"id\":0,\"instructions\":[" +
        "{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":7}}," +
        "{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":7}}," +
        "{\"op\":\"compare\",\"dst\":3,\"cmp\":\"Eq\",\"lhs\":1,\"rhs\":2}," +
        "{\"op\":\"branch\",\"cond\":3,\"then\":1,\"else\":2}]}," +
      "{\"id\":1,\"instructions\":[{\"op\":\"phi\",\"dst\":4,\"pred\":0,\"value\":1},{\"op\":\"ret\",\"value\":4}]}," +
      "{\"id\":2,\"instructions\":[{\"op\":\"ret\",\"value\":0}]}]}]}"
    local v = HakoruneVmEntryBox.run_min(j)
    print(HakoruneVmEntryBox.int_to_str(v))
    return 0
  }
}
EOF_NY

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="7"
compare_outputs "$expected" "$out" "hakorune_vm_m3_compare_branch_phi_entry_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
