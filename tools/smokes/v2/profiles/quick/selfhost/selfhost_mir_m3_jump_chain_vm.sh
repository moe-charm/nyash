#!/bin/bash
# selfhost_mir_m3_jump_chain_vm.sh — jump chain 0->1->2; final ret value preserved

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_mir_m3_jump_chain_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.entry as MiniVmEntryBox

static box Main {
  main() {
    // block0: const 5 -> r1; jump 1
    // block1: jump 2
    // block2: ret r1 (expect 5)
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":["
    j = j + "{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":5}},{\"op\":\"jump\",\"target\":1}]},"
    j = j + "{\"id\":1,\"instructions\":[{\"op\":\"jump\",\"target\":2}]},"
    j = j + "{\"id\":2,\"instructions\":[{\"op\":\"ret\",\"value\":1}]}]}]}"
    local v = MirVmMin._run_min(j)
    print(MiniVmEntryBox.int_to_str(v))
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="5"
compare_outputs "$expected" "$out" "selfhost_mir_m3_jump_chain_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

