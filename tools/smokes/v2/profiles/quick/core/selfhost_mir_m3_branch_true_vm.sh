#!/bin/bash
# selfhost_mir_m3_branch_true_vm.sh — branch(cond) selects then-path

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

# Enabled: Mini‑VM branch/jump basic

# Dev-time guards
export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1
export NYASH_BUILDER_REWRITE_INSTANCE=1

TMP_DIR="/tmp/selfhost_mir_m3_branch_true_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    // blocks: 0 -> branch(cond=1) then:1 else:2; 1: ret 1; 2: ret 2
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":["
    j = j + "{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":1}},{\"op\":\"branch\",\"cond\":1,\"then\":1,\"else\":2}]},"
    j = j + "{\"id\":1,\"instructions\":[{\"op\":\"ret\",\"value\":1}]},"
    j = j + "{\"id\":2,\"instructions\":[{\"op\":\"ret\",\"value\":2}]}]}]}"
    local v = MirVmMin._run_min(j)
    print(MirVmMin._int_to_str(v))
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="1"
compare_outputs "$expected" "$out" "selfhost_mir_m3_branch_true_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
