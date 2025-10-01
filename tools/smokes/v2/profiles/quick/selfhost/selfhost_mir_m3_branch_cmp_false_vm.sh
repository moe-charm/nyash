#!/bin/bash
# selfhost_mir_m3_branch_cmp_false_vm.sh — branch(cond from compare=false) selects else-path

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_mir_m3_branch_cmp_false_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    // bb0: const 2 -> 1; const 5 -> 2; compare Gt 1,2 -> 3; branch(cond=3, then=1, else=2)
    // bb1: ret 1; bb2: ret 0
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":["
    j = j + "{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":2}},{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":5}},{\"op\":\"compare\",\"cmp\":\"Gt\",\"lhs\":1,\"rhs\":2,\"dst\":3},{\"op\":\"branch\",\"cond\":3,\"then\":1,\"else\":2}]},"
    j = j + "{\"id\":1,\"instructions\":[{\"op\":\"ret\",\"value\":1}]},"
    j = j + "{\"id\":2,\"instructions\":[{\"op\":\"ret\",\"value\":0}]}]}]}"
    local v = MirVmMin._run_min(j)
    print(MirVmMin._int_to_str(v))
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="0"
compare_outputs "$expected" "$out" "selfhost_mir_m3_branch_cmp_false_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

