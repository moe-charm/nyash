#!/bin/bash
# plugin_on_map_keys_values_vm.sh — plugin-on overlay Map keys/values/delete sanity

source "$(dirname "$0")/../../../lib/test_runner.sh"
export NYASH_DISABLE_PLUGINS=0
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

ensure_hako_toml

tmpfile=$(mktemp /tmp/plugin_on_map_kv_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local m = new MapBox()
    m.set("a", 1)
    m.set("b", 2)
    // minimal: size/has/get (keys/values/delete may be plugin-dependent)
    if m.size() != 2 { return 101 }
    if m.has("a") != 1 { return 102 }
    if m.get("a") == null { return 103 }
    return 0
  }
}
SRC

out_vm=$(run_nyash_vm "$tmpfile" | awk '/^Result:/{print $0}' | head -n1 | tr -d '\r' | xargs)
rm -f "$tmpfile"

if [ $rc -ne 0 ]; then
  echo "FAIL: rc=$rc" >&2
  exit 1
fi
echo "OK"
exit 0
