#!/bin/bash
# selfhost_pipeline_v2_op_eq_vm.sh — P3 minimal: Extern op_eq via mir_call (rc-only)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

export NYASH_DEV=1
export NYASH_ALLOW_USING_FILE=1

TMP_DIR="/tmp/selfhost_pipeline_v2_op_eq_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'EOF'
using selfhost.vm.entry as MiniVmEntryBox
using "selfhost/compiler/pipeline_v2/emit_mir_flow.hako" as EmitMirFlow

static box Main {
  main() {
    // 7 == 7 → expect true; rc-only
    local j = EmitMirFlow.emit_op_eq(7, 7)
    return MirVmMin.run(j)
  }
}
EOF

"$NYASH_BIN" --backend vm "$TMP_DIR/driver.nyash" >/dev/null 2> >(filter_noise 1>&2)
rc=$?
rm -rf "$TMP_DIR"
exit $rc

