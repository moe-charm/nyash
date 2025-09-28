#!/bin/bash
# selfhost_mir_m2_ret_first_vm.sh — ret at block head (value undefined → treated as 0)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_mir_m2_ret_first_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    // Block 0: ret 1; const 1 (unreached for result) → expected 0 (undefined treated as 0 in MirVmMin)
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":["
    j = j + "{\"id\":0,\"instructions\":[{\"op\":\"ret\",\"value\":1},{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":42}}]}]}]}"
    local v = MirVmMin._run_min(j)
    print(MirVmMin._int_to_str(v))
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="0"
compare_outputs "$expected" "$out" "selfhost_mir_m2_ret_first_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

