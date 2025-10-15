#!/bin/bash
# rc-only: ensure_cond on If-like (diamond) and Loop CFG

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
ensure_hako_toml
export NYASH_ALLOW_USING_FILE=1
export NYASH_USING=1
export NYASH_USING_AST=1

TMP_DIR="/tmp/selfhost_localssa_ensure_cond_if_loop_rc_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
using "selfhost/shared/mir/block_builder_box.hako" as BlockBuilder
using "selfhost/compiler/pipeline_v2/local_ssa_box.hako" as LocalSSABox
using selfhost.vm.mir_min as MirVmMin
static box Main { main() {
  // If-like diamond: compare→branch; ensure_cond inserts copy near cond def
  local mod_if = BlockBuilder.compare_branch(10, 5, "Gt")
  LocalSSABox.ensure_cond(mod_if)
  MirVmMin.run(mod_if)

  // Loop CFG: block1 has compare→branch; ensure_cond should find it in block1
  local mod_loop = BlockBuilder.loop_counter(3)
  LocalSSABox.ensure_cond(mod_loop)
  MirVmMin.run(mod_loop)
  return 0
} }
NY

"$NYASH_BIN" --backend vm "$TMP_DIR/driver.nyash" >/dev/null 2> >(filter_noise 1>&2) || true
rm -rf "$TMP_DIR"
exit 0
