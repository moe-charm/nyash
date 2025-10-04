#!/bin/bash
# selfhost_minivm_thin_vs_legacy_compare_ret_vm.sh — legacy vs thin should match when ret is compare result

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export SMOKES_TIMEOUT_SEC=${SMOKES_TIMEOUT_SEC:-25}
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_minivm_thin_vs_legacy_compare_ret_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    // const 3->1, const 3->2, Eq(1,2)->3, ret 3 → expect 1 on both modes
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":[{\"id\":0,\"instructions\":["
    j = j + "{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":3}},"
    j = j + "{\"op\":\"const\",\"dst\":2,\"value\":{\"type\":\"i64\",\"value\":3}},"
    j = j + "{\"op\":\"compare\",\"dst\":3,\"cmp\":\"Eq\",\"lhs\":1,\"rhs\":2},"
    j = j + "{\"op\":\"ret\",\"value\":3}]}]}]}"
    local v1 = MirVmMin._run_min(j)
    print(MirVmMin._int_to_str(v1))
    local v2 = MirVmMin.run_thin(j)
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | grep -E '^-?[0-9]+$' | tail -n 2 | tr -d '\r' | xargs echo)
expected="1 1"
compare_outputs "$expected" "$out" "selfhost_minivm_thin_vs_legacy_compare_ret_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
