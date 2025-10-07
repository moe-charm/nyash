#!/bin/bash
# using_private_warn_vm.sh — private patterns warn mode (no strict)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
# Force private patterns for the test to avoid env drift
export NYASH_PRIVATE_PATTERNS='apps/selfhost-compiler/pipeline_v2/emit_compare_box.hako'
export NYASH_PRIVATE_DIAG=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/using_private_warn_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/driver.nyash"

cat > "$SRC" << 'NYEOF'
using "apps/selfhost-compiler/pipeline_v2/emit_compare_box.hako" as EC
static box Main { main() { return 0 } }
NYEOF

out=$(run_nyash_vm "$SRC" --dev 2>&1)
echo "$out" | grep -q '"code":"private_access"' || { echo "$out"; rm -rf "$TMP_DIR"; exit 1; }

rm -rf "$TMP_DIR"
exit 0
