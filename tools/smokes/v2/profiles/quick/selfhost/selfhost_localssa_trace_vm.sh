#!/bin/bash
# selfhost_localssa_trace_vm.sh — Verify LocalSSA trace API reports copy insertions
# tags: selfhost,trace

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_localssa_trace_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using "apps/selfhost-compiler/builder/ssa/local.hako" as LocalSSA

static box Main {
  main() {
    // Make cond come from previous block to force a copy
    local j = "{\"functions\":[{\"name\":\"main\",\"params\":[],\"blocks\":["
    j = j + "{\"id\":0,\"instructions\":[{\"op\":\"const\",\"dst\":1,\"value\":{\"type\":\"i64\",\"value\":1}},{\"op\":\"jump\",\"target\":1}]},"
    j = j + "{\"id\":1,\"instructions\":[{\"op\":\"branch\",\"cond\":1,\"then\":2,\"else\":3}]},"
    j = j + "{\"id\":2,\"instructions\":[{\"op\":\"ret\",\"value\":1}]},"
    j = j + "{\"id\":3,\"instructions\":[{\"op\":\"ret\",\"value\":0}]}]}]}"
    LocalSSA.trace_enable(1)
    local out = LocalSSA.ensure_cond(j)
    print(LocalSSA.trace_summary_after(out))
    return 0
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
# Expect at least one branch and one copy
echo "$out" | grep -q "branches=" || { echo "no branches in trace" >&2; exit 1; }
echo "$out" | grep -q "copies=" || { echo "no copies in trace" >&2; exit 1; }
val=$(echo "$out" | sed -n 's/.*copies=\([0-9]\+\).*/\1/p')
test "${val:-0}" -ge 1 || { echo "expected copies>=1, got: $val" >&2; exit 1; }

rm -rf "$TMP_DIR"
exit 0
