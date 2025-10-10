#!/bin/bash
# plugin_on_map_keys_values_handle_vm.sh — Stage-2: Map.keys/values return HostHandle(ArrayBox)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export NYASH_DISABLE_PLUGINS=0
export SMOKES_USE_PYVM=0
export HAKO_EXPORT_HOST=1
export NYASH_PLUGIN_MAP_ARRAY_HANDLE=1
require_env || exit 2
preflight_plugins || exit 2

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
    if ks.length() != 2 { return 901 }
    local vs = m.values()
    if vs.length() != 2 { return 903 }
    return 0
  }
}
SRC

out_vm=$(run_nyash_vm "$tmpfile" | awk '/^Result:/{print $0}' | head -n1 | tr -d '\r' | xargs)
rm -f "$tmpfile"

if [ "$out_vm" != "Result: 0" ]; then
  echo "FAIL: expected 'Result: 0', got '$out_vm'" >&2
  exit 1
fi
echo "$out_vm"
exit 0
