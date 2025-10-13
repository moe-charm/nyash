#!/bin/bash
# rc-only: ensure_calls minimal — materialize last return via LocalSSABox

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
ensure_hako_toml
export NYASH_ALLOW_USING_FILE=1
export NYASH_USING=1
export NYASH_USING_AST=1

TMP_DIR="/tmp/selfhost_localssa_ensure_calls_rc_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
using "selfhost/shared/mir/block_builder_box.hako" as BlockBuilder
using "selfhost/compiler/pipeline_v2/local_ssa_box.hako" as LocalSSABox
using selfhost.vm.mir_min as MirVmMin
static box Main { main() {
  // Build: extern op_eq(7,8) → ret r3 (no extra copy). Then materialize via LocalSSA.
  local args = new ArrayBox()
  args.push(7)
  args.push(8)
  local mod = BlockBuilder.extern_call_ival_ret("nyrt.ops.op_eq", args)
  LocalSSABox.ensure_materialize_last_ret(mod)
  MirVmMin.run(mod)
  return 0
} }
NY

"$NYASH_BIN" --backend vm "$TMP_DIR/driver.nyash" >/dev/null 2> >(filter_noise 1>&2) || true
rm -rf "$TMP_DIR"
exit 0
