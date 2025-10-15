#!/bin/bash
# plugin_on_p4_emit_calls_rc_vm.sh — rc-only: P4 thin adapters under plugin-on overlay

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_PROFILE_ENV=plugin-on
export NYASH_DISABLE_PLUGINS=0
export SMOKES_DISABLE_PLUGIN_CHECKS=0

require_env || exit 2
preflight_plugins || exit 2
ensure_hako_toml

# Precheck: NewBox ArrayBox must succeed with plugins
precheck_src=$(mktemp /tmp/plugin_on_p4_pre_XXXX.hako)
cat >"$precheck_src" << 'SRC'
static box Main { main() { local a = new ArrayBox(); return 0 } }
SRC
run_nyash_vm "$precheck_src" >/dev/null || { echo 'SKIP: plugins not available' >&2; rm -f "$precheck_src"; exit 0; }
rm -f "$precheck_src"

TMP_DIR="/tmp/plugin_on_p4_calls_rc_$$"
mkdir -p "$TMP_DIR"

cat > "$TMP_DIR/driver.nyash" << 'NY'
using "selfhost/compiler/pipeline_v2/emit_mir_flow.hako" as Emit
using selfhost.vm.mir_min as MirVmMin
static box Main { main() {
  // plugin constructors should resolve via provider/loader
  local j = Emit.emit_constructor("ArrayBox", new ArrayBox())
  MirVmMin.run(j)
  // Then method size() through P4 thin wrapper
  local j2 = Emit.emit_method_call("size", 0, new ArrayBox())
  MirVmMin.run(j2)
  return 0
} }
NY

"$NYASH_BIN" --backend vm "$TMP_DIR/driver.nyash" >/dev/null 2> >(filter_noise 1>&2) || { rm -rf "$TMP_DIR"; exit 1; }
rm -rf "$TMP_DIR"
exit 0
