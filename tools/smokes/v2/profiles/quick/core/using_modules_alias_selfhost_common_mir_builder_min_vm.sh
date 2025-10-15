#!/bin/bash
# using_modules_alias_selfhost_common_mir_builder_min_vm.sh — [workspace] resolve-only: selfhost.common.json.mir_builder_min

source "$(dirname "$0")/../../../lib/test_runner.sh"
# Gate: this module emits MIR and may be under active development; keep SKIP by default
if [ "${SMOKES_ENABLE_MIR_BUILDER_MIN:-0}" != "1" ]; then
  log_warn "SKIP using_modules_alias_selfhost_common_mir_builder_min_vm (set SMOKES_ENABLE_MIR_BUILDER_MIN=1 to run)"
  exit 0
fi
export SMOKES_DISABLE_PLUGIN_CHECKS=1
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

TMP_DIR="/tmp/using_modules_alias_selfhost_common_mir_builder_min_vm_$$"
mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/main.nyash"

cat > "$SRC" << 'NYEOF'
using selfhost.common.json.mir_builder_min as MirJsonBuilderMin

static box Main {
  main() {
    print("ok")
    return 0
  }
}
NYEOF

out=$(run_nyash_vm "$SRC" | tail -n 1 | tr -d '\r' | xargs)
if [ "$out" = "ok" ]; then
  log_success "using_modules_alias_selfhost_common_mir_builder_min_vm resolved workspace manifest"
  rm -rf "$TMP_DIR"
  exit 0
else
  log_error "using_modules_alias_selfhost_common_mir_builder_min_vm expected ok, got: ${out:-<empty>}"
  rm -rf "$TMP_DIR"
  exit 1
fi
