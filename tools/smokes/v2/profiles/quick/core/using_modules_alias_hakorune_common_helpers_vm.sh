#!/bin/bash
# using_modules_alias_hakorune_common_helpers_vm.sh — [workspace] E2E: hakorune.common.strings.helpers

source "$(dirname "$0")/../../../lib/test_runner.sh"
# Gate: delegate chain may depend on alias resolution; keep SKIP by default
if [ "${SMOKES_ENABLE_HAKORUNE_HELPERS:-0}" != "1" ]; then
  log_warn "SKIP using_modules_alias_hakorune_common_helpers_vm (set SMOKES_ENABLE_HAKORUNE_HELPERS=1 to run)"
  exit 0
fi
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/using_modules_alias_hakorune_common_helpers_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'NYEOF'
using hakorune.common.strings.helpers as HakoruneStringHelpers

static box Main {
  main() {
    local s = HakoruneStringHelpers.int_to_str(7)
    if (s == "7") { print("ok") } else { print("ng") }
    return 0
  }
}
NYEOF

out=$(run_nyash_vm "$SRC" | tail -n 1 | tr -d '\r' | xargs)
if [ "$out" = "ok" ]; then
  log_success "using_modules_alias_hakorune_common_helpers_vm resolved workspace manifest"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "using_modules_alias_hakorune_common_helpers_vm expected ok, got: ${out:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi
