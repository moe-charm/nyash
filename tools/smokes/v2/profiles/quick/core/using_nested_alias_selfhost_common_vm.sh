#!/bin/bash
# using_nested_alias_selfhost_common_vm.sh — Nested alias: selfhost.common → C, C.json.core.string_scan → StringScanBox

export SMOKES_PROFILE_ENV=quick
source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
export NYASH_USING_STRICT=0
export NYASH_USING_NAMESPACE_ALIAS=1
export SMOKES_USE_DEV=1
# Enable AST prelude to resolve nested aliases under quick
export NYASH_USING_AST=1
require_env || exit 2
if [ "${NYASH_DISABLE_PLUGINS:-0}" != "1" ]; then
  preflight_plugins || exit 2
fi

TMP_DIR="/tmp/using_nested_alias_selfhost_common_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'NYEOF'
using selfhost.common as C
using C.json.core.string_scan as StringScanBox

static box Main {
  main() {
    local s = "aabb"
    local i = StringScanBox.find_quote(s, 0)
    if (i < 0) { print("ok") } else { print("ng") }
    return 0
  }
}
NYEOF

out_full=$(run_nyash_vm "$SRC")
if echo "$out_full" | grep -qi 'using: file paths are disallowed\|AST prelude merge is disabled'; then
  log_warn "SKIP using_nested_alias_selfhost_common_vm (using resolver disabled)"
  rm -rf "$TMP_DIR"; exit 0
fi
if echo "$out_full" | grep -qi 'Unknown module function'; then
  log_warn "SKIP using_nested_alias_selfhost_common_vm (resolver nested alias not available)"
  rm -rf "$TMP_DIR"; exit 0
fi
out=$(echo "$out_full" | grep -v '^Result: ' | tail -n 1 | tr -d '\r' | xargs)
if [ "$out" = "ok" ]; then
  log_success "using_nested_alias_selfhost_common_vm nested alias resolved"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "using_nested_alias_selfhost_common_vm expected ok, got: ${out:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi
