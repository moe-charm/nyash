#!/bin/bash
# selfhost_mir_m3_branch_cmp_true_vm.sh — branch(cond from compare=true) selects then-path

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_mir_m3_branch_cmp_true_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.entry as MiniVmEntryBox

static box Main {
  main() {
    // bb0: const 7 -> 1; const 4 -> 2; compare Gt 1,2 -> 3; branch(cond=3, then=1, else=2)
    // bb1: ret 3 (compare result=1); bb2: ret 0
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":["
    j = j + "{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":7}},{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":4}},{\"op\":\"compare\",\"cmp\":\"Gt\",\"lhs\":1,\"rhs\":2,\"dst\":3},{\"op\":\"branch\",\"cond\":3,\"then\":1,\"else\":2}]},"
    j = j + "{\"id\":1,\"instructions\":[{\"op\":\"ret\",\"value\":3}]},"
    j = j + "{\"id\":2,\"instructions\":[{\"op\":\"ret\",\"value\":0}]}]}]}"
    local v = MiniVmEntryBox.run_min(j)
    print(MiniVmEntryBox.int_to_str(v))
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="1"
compare_outputs "$expected" "$out" "selfhost_mir_m3_branch_cmp_true_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
