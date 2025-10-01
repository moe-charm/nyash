#!/bin/bash
# selfhost_mir_m3_branch_cond_prev_block_vm.sh — branch(cond) uses value defined in previous block
# tags: selfhost

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_mir_m3_branch_cond_prev_block_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    // block0: const 1->r1 (cond); const 9->r9; jump 1
    // block1: branch(cond=r1) then:2 else:3
    // block2: ret r9 (9)
    // block3: ret r0 (0, default)
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":["
    j = j + "{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":1}},{\"op\":\"const\",\"dst\":9,\"value\":{\"type\":\"i64\",\"value\":9}},{\"op\":\"jump\",\"target\":1}]},"
    j = j + "{\"id\":1,\"instructions\":[{\"op\":\"branch\",\"cond\":1,\"then\":2,\"else\":3}]},"
    j = j + "{\"id\":2,\"instructions\":[{\"op\":\"ret\",\"value\":9}]},"
    j = j + "{\"id\":3,\"instructions\":[{\"op\":\"ret\",\"value\":0}]}]}]}"
    local v = MirVmMin._run_min(j)
    print(MirVmMin._int_to_str(v))
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="9"
compare_outputs "$expected" "$out" "selfhost_mir_m3_branch_cond_prev_block_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
