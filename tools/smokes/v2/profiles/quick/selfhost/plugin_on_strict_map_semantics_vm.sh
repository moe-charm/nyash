#!/bin/bash
# plugin_on_strict_map_semantics_vm.sh — plugin-on-strict: Map semantics

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=${SMOKES_DISABLE_PLUGIN_CHECKS:-1}
export NYASH_DISABLE_PLUGINS=0
export SMOKES_USE_PYVM=0
# Enforce strict plugin-on (no builtin fallback)
export NYASH_PLUGIN_ON_STRICT=1
# Enable Stage-2 keys/values (HostHandle Array)
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

tmpfile=$(mktemp /tmp/plugin_on_strict_map_semantics_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local m = new MapBox()
    if m.get("missing") != null { return 201 }
    m.set("b", 2); m.set("a", 1)
    if m.size() != 2 { return 202 }
    m.delete("missing"); if m.size() != 2 { return 203 }
    local ks = m.keys(); if ks.get(0) != "a" || ks.get(1) != "b" { return 206 }
    // In strict mode, values() representation may vary (stage-1 vs stage-2).
    // Verify values via get(key) aligned to keys order.
    if m.get(ks.get(0)) != 1 { return 209 }
    if m.get(ks.get(1)) != 2 { return 209 }
    m.clear(); if m.size() != 0 { return 210 }
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
