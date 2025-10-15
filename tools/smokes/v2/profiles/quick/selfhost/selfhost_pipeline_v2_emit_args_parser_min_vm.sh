#!/bin/bash
# selfhost_pipeline_v2_emit_args_parser_min_vm.sh — Minimal emit verification
# Goal: Ensure Stage1ArgsParserBox-backed emit materializes the right number of consts.

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
export SMOKES_DISABLE_PLUGIN_CHECKS=1
require_env || exit 2
preflight_plugins || exit 2
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi

TMP_DIR="/tmp/selfhost_pipeline_v2_emit_args_parser_min_vm_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NYEOF'
using "selfhost/compiler/pipeline_v2/stage1_args_parser_box.hako" as Stage1ArgsParserBox

static box Main {
  main() {
    // Args parser: ensure 3 ints are parsed from "[1,2,3]"
    local arr = Stage1ArgsParserBox.parse_ints("[1,2,3]")
    local n = arr.size()
    print("" + n)
    return n
  }
}

NYEOF
out=$(run_nyash_vm "$TMP_DIR/driver.nyash" --dev | tail -n 1 | tr -d r | xargs)
expected="3"
compare_outputs "$expected" "$out" "selfhost_pipeline_v2_emit_args_parser_min_vm" || { rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
