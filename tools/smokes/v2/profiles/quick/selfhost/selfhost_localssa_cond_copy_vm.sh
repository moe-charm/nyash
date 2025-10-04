#!/bin/bash
# selfhost_localssa_cond_copy_vm.sh — LocalSSA.ensure_cond inserts copy when cond is defined in a previous block

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_localssa_cond_copy_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "apps/selfhost-compiler/builder/ssa/cond_inserter.hako" as CondInserter

static box Main {
  main() {
    // block0: const 1->r1; jump 1
    // block1: branch(cond=r1) then:2 else:3   (cond from prev block)
    // block2: ret r1
    // block3: ret 0
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":["
    j = j + "{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":1}},{\"op\":\"jump\",\"target\":1}]},"
    j = j + "{\"id\":1,\"instructions\":[{\"op\":\"branch\",\"cond\":1,\"then\":2,\"else\":3}]},"
    j = j + "{\"id\":2,\"instructions\":[{\"op\":\"ret\",\"value\":1}]},"
    j = j + "{\"id\":3,\"instructions\":[{\"op\":\"ret\",\"value\":0}]}]}]}"
    local out = CondInserter.ensure_cond(j)
    // simple contains check (both plain and escaped patterns)
    local has = 0
    // plain JSON pattern
    if out.indexOf("\"op\":\"copy\"") >= 0 { has = 1 }
    // escaped JSON pattern (when JSON text is double-escaped)
    if out.indexOf("\\\"op\\\":\\\"copy\\\"") >= 0 { has = 1 }
    if has == 1 { print("copy:1") } else { print("copy:0") }
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="copy:1"
compare_outputs "$expected" "$out" "selfhost_localssa_cond_copy_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
