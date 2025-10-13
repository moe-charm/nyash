#!/bin/bash
# plugin_on_map_keys_values_handle_vm.sh — Stage-2: Map.keys/values return HostHandle(ArrayBox)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=${SMOKES_DISABLE_PLUGIN_CHECKS:-1}
export NYASH_DISABLE_PLUGINS=0
export SMOKES_USE_PYVM=0
export HAKO_EXPORT_HOST=1
export NYASH_PLUGIN_MAP_ARRAY_HANDLE=1
require_env || exit 2
preflight_plugins || exit 2
precheck_src=$(mktemp /tmp/plugin_on_pre_XXXX.hako)
cat >"$precheck_src" << 'SRC'
static box Main { main() { local m = new MapBox(); return 0 } }
SRC
run_nyash_vm "$precheck_src" >/dev/null
pre_rc=$?
rm -f "$precheck_src"
if [ $pre_rc -ne 0 ]; then
  echo "SKIP: plugins not available (precheck rc=$pre_rc)" >&2
  exit 0
fi

ensure_hako_toml

# Skip when host export symbols are not present in the binary (requires build with HAKO_EXPORT_HOST=1)
if ! strings "$NYASH_BIN" 2>/dev/null | grep -q 'nyrt_host_call_slot'; then
  log_warn "SKIP plugin_on_map_keys_values_handle_vm (host export symbols missing; build with HAKO_EXPORT_HOST=1)"
  exit 0
fi

tmpfile=$(mktemp /tmp/plugin_on_map_kv_handle_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local m = new MapBox()
    m.set("b", 1)
    m.set("a", 2)
    local ks = m.keys()
    if ks.size() != 2 { return 901 }
    local vs = m.values()
    if vs.size() != 2 { return 903 }
    return 0
  }
}
SRC

out_vm=$(run_nyash_vm "$tmpfile" )
rc=$?
rm -f "$tmpfile"

rm -f "$tmpfile"
if [ $rc -ne 0 ]; then echo "FAIL: rc=$rc" >&2; exit 1; fi
echo OK
exit 0
