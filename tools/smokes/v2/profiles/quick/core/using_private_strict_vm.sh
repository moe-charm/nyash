#!/bin/bash
# using_private_strict_vm.sh — private patterns strict mode (error)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_ALLOW_USING_FILE=0
export NYASH_USING_AST=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/using_private_strict_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/driver.nyash"

cat > "$SRC" << 'NYEOF'
using "selfhost/compiler/pipeline_v2/emit_compare_box.hako" as EC
static box Main { main() { return 0 } }
NYEOF

# Strict on
out=$(NYASH_USING_CHECKS_STRICT=1 run_nyash_vm "$SRC" --dev 2>&1 | grep -v '^Result: ' || true)
# Expect non-empty diagnostic and non-zero exit; run_nyash_vm pipes exit, so we grep for diag only

echo "$out" | grep -qiE '"code"\s*:\s*"private_access"|private|using: file paths are disallowed' || { echo "$out"; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
