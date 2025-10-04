#!/bin/bash
# selfhost_mir_m2_multi_compare_last_ret_vm.sh — Multi-compare in same block, ret uses last compare (mix v0/v1)
# tags: selfhost

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_TIMEOUT_SEC=${SMOKES_TIMEOUT_SEC:-25}
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_mir_m2_multi_compare_last_ret_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    // Block 0: const 7 -> r1, const 8 -> r2, compare Eq(r1,r2) -> r3 (false),
    //          compare (v1 form) operation:"==" lhs:1 rhs:1 -> r4 (true), ret r4
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":["
    j = j + "{\"id\":0,\"instructions\":["
    j = j + "{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":7}},"
    j = j + "{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":8}},"
    j = j + "{\"op\":\"compare\",\"dst\":3,\"cmp\":\"Eq\",\"lhs\":1,\"rhs\":2},"
    j = j + "{\"op\":\"compare\",\"dst\":4,\"operation\":\"==\",\"lhs\":1,\"rhs\":1},"
    j = j + "{\"op\":\"ret\",\"value\":4}]}]}]}"
    local v = MirVmMin._run_min(j)
    print(MirVmMin._int_to_str(v))
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="1"
compare_outputs "$expected" "$out" "selfhost_mir_m2_multi_compare_last_ret_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
