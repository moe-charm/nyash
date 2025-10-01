#!/bin/bash
# selfhost_localssa_equivalence_vm.sh — ensure LocalSSA.ensure_cond preserves semantics (equivalence)
# tags: selfhost

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_localssa_equivalence_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "apps/selfhost-compiler/builder/ssa/local.nyash" as LocalSSA
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    // Original MIR: cond comes from previous block (will trigger LocalSSA copy)
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":["
    j = j + "{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":1}},{\"op\":\"jump\",\"target\":1}]},"
    j = j + "{\"id\":1,\"instructions\":[{\"op\":\"branch\",\"cond\":1,\"then\":2,\"else\":3}]},"
    j = j + "{\"id\":2,\"instructions\":[{\"op\":\"ret\",\"value\":1}]},"
    j = j + "{\"id\":3,\"instructions\":[{\"op\":\"ret\",\"value\":0}]}]}]}"

    local v0 = MirVmMin._run_min(j)
    local jj = LocalSSA.ensure_cond(j)
    local v1 = MirVmMin._run_min(jj)
    // print both results to compare outside
    print(MirVmMin._int_to_str(v0) + "," + MirVmMin._int_to_str(v1))
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="1,1"
compare_outputs "$expected" "$out" "selfhost_localssa_equivalence_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
