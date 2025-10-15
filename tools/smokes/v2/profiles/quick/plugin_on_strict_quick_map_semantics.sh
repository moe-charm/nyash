#!/bin/bash
# quick profile wrapper — plugin-on-strict map semantics minimal
source "$(dirname "$0")/../../lib/test_runner.sh"
export SMOKES_PROFILE_ENV=plugin-on-strict
require_env || exit 2
preflight_plugins || { echo "SKIP: plugins not available (preflight)" >&2; exit 0; }
ensure_hako_toml

# Try to (re)build plugins via plugin-tester when available (strict needs real .so)
if [ -x "$NYASH_ROOT/tools/plugin-tester/target/release/plugin-tester" ]; then
  "$NYASH_ROOT/tools/plugin-tester/target/release/plugin-tester" build-all >/dev/null 2>&1 || { echo "SKIP: plugin build-all failed" >&2; exit 0; }
fi

# Skip gracefully when MapBox cannot be constructed (plugins missing under strict policy)
precheck_src=$(mktemp /tmp/strict_quick_map_pre_XXXX.hako)
cat >"$precheck_src" << 'SRC'
static box Main {
  main() {
    local m = new MapBox()
    return 0
  }
}
SRC
run_nyash_vm "$precheck_src" >/dev/null
pre_rc=$?
rm -f "$precheck_src"
if [ $pre_rc -ne 0 ]; then
  echo "SKIP: MapBox not available under strict plugin policy (precheck rc=$pre_rc)" >&2
  exit 0
fi

# Main check (order-independent, minimal semantics)
tmpfile=$(mktemp /tmp/strict_quick_map_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local m = new MapBox()
    if m.get("x") != null { return 1 }
    m.set("b", 2)
    m.set("a", 1)
    local ks = m.keys()
    if ks == null { return 2 }
    if ks.size() != 2 { return 3 }
    return 0
  }
}
SRC
run_nyash_vm "$tmpfile" >/dev/null
rc=$?
rm -f "$tmpfile"
if [ $rc -ne 0 ]; then
  echo "FAIL: rc=$rc" >&2
  exit 1
fi
echo "OK"
exit 0
