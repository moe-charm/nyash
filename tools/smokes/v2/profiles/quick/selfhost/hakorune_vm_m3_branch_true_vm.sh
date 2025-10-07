#!/bin/bash
# hakorune_vm_m3_branch_true_vm.sh — branch(cond) selects then-path (Hakorune)
# tags: selfhost hakorune

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1
export NYASH_BUILDER_REWRITE_INSTANCE=1

TMP_DIR="/tmp/hakorune_vm_m3_branch_true_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF_NY'
using hakorune.vm.entry as HakoruneVmEntryBox

static box Main {
  main() {
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":["
    j = j + "{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":1}},{\"op\":\"branch\",\"cond\":1,\"then\":1,\"else\":2}]},"
    j = j + "{\"id\":1,\"instructions\":[{\"op\":\"ret\",\"value\":1}]},"
    j = j + "{\"id\":2,\"instructions\":[{\"op\":\"ret\",\"value\":2}]}]}]}"
    local v = HakoruneVmEntryBox.run_min(j)
    print(HakoruneVmEntryBox.int_to_str(v))
    return 0
  }
}
EOF_NY

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="1"
compare_outputs "$expected" "$out" "hakorune_vm_m3_branch_true_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
