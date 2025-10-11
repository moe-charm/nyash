#!/bin/bash
# selfhost_pipeline_v2_blockbuilder_calls_vm.sh — rc-only: P4 BlockBuilder call emitters (ctor/extern)

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
export NYASH_ALLOW_USING_FILE=1
export NYASH_USING=1
export NYASH_USING_AST=1
ensure_hako_toml

TMP_DIR="/tmp/selfhost_pipeline_v2_blockbuilder_calls_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
using "selfhost/compiler/pipeline_v2/emit_mir_flow.hako" as Emit
using selfhost.vm.mir_min as MirVmMin

static box Main {
  main() {
    // Extern(op_eq false) のJSONを1本生成し、MirVmMinで直接実行
    local j = Emit.emit_op_eq(7, 8)
    MirVmMin.run(j)
    return 0
  }
}
NY

pushd "$TMP_DIR" >/dev/null
"$NYASH_BIN" --backend vm driver.nyash >/dev/null 2> >(filter_noise 1>&2) || { popd >/dev/null; rm -rf "$TMP_DIR"; exit 1; }

popd >/dev/null
rm -rf "$TMP_DIR"
exit 0
