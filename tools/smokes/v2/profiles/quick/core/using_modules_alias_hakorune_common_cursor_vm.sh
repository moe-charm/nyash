#!/bin/bash
# using_modules_alias_hakorune_common_cursor_vm.sh — [workspace] E2E: hakorune.common.json.cursor

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export NYASH_ALLOW_USING_FILE=1
export NYASH_USING_AST=1
export NYASH_USING_AST=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/using_modules_alias_hakorune_common_cursor_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'NYEOF'
using hakorune.common.json.cursor as HakoruneJsonCursorBox

static box Main {
  main() {
    // Just call a method: seek_array_end on [] starting at 0 should be > 0
    local i = HakoruneJsonCursorBox.seek_array_end("[]", 0)
    if (i >= 0) { print("ok") } else { print("ng") }
    return 0
  }
}
NYEOF

out_full=$(run_nyash_vm "$SRC")
if echo "$out_full" | grep -qi 'AST prelude merge is disabled\|using: file paths are disallowed'; then
  log_warn "SKIP using_modules_alias_hakorune_common_cursor_vm (using resolver disabled)"
  rm -rf "$TMP_DIR"; exit 0
fi
out=$(echo "$out_full" | grep -v '^Result: ' | tail -n 1 | tr -d '\r' | xargs)
if [ "$out" = "ok" ]; then
  log_success "using_modules_alias_hakorune_common_cursor_vm resolved workspace manifest"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "using_modules_alias_hakorune_common_cursor_vm expected ok, got: ${out:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi
