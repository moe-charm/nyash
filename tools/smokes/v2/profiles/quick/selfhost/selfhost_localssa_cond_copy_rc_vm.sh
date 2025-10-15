#!/bin/bash
# selfhost_localssa_cond_copy_rc_vm.sh — rc-only: ensure_cond(materialize=1) path via EmitMirFlow.emit_compare_cfg2

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
ensure_hako_toml
export NYASH_ALLOW_USING_FILE=1
export NYASH_USING=1
export NYASH_USING_AST=1

TMP_DIR="/tmp/selfhost_localssa_cond_copy_rc_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
using "selfhost/compiler/pipeline_v2/emit_mir_flow.hako" as Emit
using selfhost.vm.mir_min as MirVmMin
static box Main { main() {
  local j = Emit.emit_compare_cfg2(7, 3, "Gt", 1)
  MirVmMin.run(j)
  return 0
} }
NY

"$NYASH_BIN" --backend vm "$TMP_DIR/driver.nyash" >/dev/null 2> >(filter_noise 1>&2) || { rm -rf "$TMP_DIR"; exit 1; }
rm -rf "$TMP_DIR"
exit 0
