#!/bin/bash
# selfhost_pipeline_v2_loop_counter_vm.sh — Emit loop counter via Pipeline V2 utility and run on Mini‑VM

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_pipeline_v2_loop_counter_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.mir_min as MirVmMin
using "apps/selfhost-compiler/pipeline_v2/emit_mir_flow.hako" as EmitMirFlow

static box Main {
  main() {
    local j = EmitMirFlow.emit_loop_counter(7)
    return MirVmMin.run(j)  // expect 7
  }
}
EOF

out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d '\r' | xargs)
expected="7"
compare_outputs "$expected" "$out" "selfhost_pipeline_v2_loop_counter_vm" || { cd /; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0

