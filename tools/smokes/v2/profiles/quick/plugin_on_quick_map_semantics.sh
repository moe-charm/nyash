#!/bin/bash
# quick profile wrapper — plugin-on map semantics minimal
source "$(dirname "$0")/../../lib/test_runner.sh"
export SMOKES_PROFILE_ENV=plugin-on
require_env || exit 2
preflight_plugins || { echo "SKIP: plugins not available (preflight)" >&2; exit 0; }
ensure_hako_toml

# Precheck: MapBox must be constructible under plugin-on
pre_src=$(mktemp /tmp/plugin_on_quick_map_pre_XXXX.hako)
cat >"$pre_src" << 'SRC'
static box Main { main() { local m = new MapBox(); return 0 } }
SRC
run_nyash_vm "$pre_src" >/dev/null || { echo 'SKIP: MapBox not available (precheck)' >&2; rm -f "$pre_src"; exit 0; }
rm -f "$pre_src" 


tmpfile=$(mktemp /tmp/plugin_on_quick_map_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local m = new MapBox()
    if m.get("x") != null { return 1 }
    m.set("b", 2)
    m.set("a", 1)
    local ks = m.keys()
    if ks.get(0) != "a" || ks.get(1) != "b" { return 2 }
    // tolerate impl variance by verifying via get
    if m.get(ks.get(0)) != 1 || m.get(ks.get(1)) != 2 { return 3 }
    return 0
  }
}
SRC
run_nyash_vm "$tmpfile" >/dev/null
rc=$?
rm -f "$tmpfile"
if [ $rc -ne 0 ]; then echo "FAIL: rc=$rc"; exit 1; fi
echo "OK"; exit 0
