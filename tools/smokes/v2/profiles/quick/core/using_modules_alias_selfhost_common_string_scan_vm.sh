#!/bin/bash
# using_modules_alias_selfhost_common_string_scan_vm.sh — [workspace] E2E: selfhost.common.json.core.string_scan

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/using_modules_alias_selfhost_common_string_scan_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'NYEOF'
using selfhost.common.json.core.string_scan as StringScanBox

static box Main {
  main() {
    local i = StringScanBox.find_quote("xx\"yy", 0)
    if (i >= 0) { print("ok") } else { print("ng") }
    return 0
  }
}
NYEOF

out=$(run_nyash_vm "$SRC" | tail -n 1 | tr -d '\r' | xargs)
if [ "$out" = "ok" ]; then
  log_success "using_modules_alias_selfhost_common_string_scan_vm resolved workspace manifest"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "using_modules_alias_selfhost_common_string_scan_vm expected ok, got: ${out:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi
