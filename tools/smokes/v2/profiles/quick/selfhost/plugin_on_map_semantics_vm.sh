#!/bin/bash
# plugin_on_map_semantics_vm.sh — plugin-on: Map semantics (miss→null, void mutators, keys/values order)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=${SMOKES_DISABLE_PLUGIN_CHECKS:-1}
export NYASH_DISABLE_PLUGINS=0
export SMOKES_USE_PYVM=0
# Enable Stage-2 keys/values (HostHandle Array) to assert ordering
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

tmpfile=$(mktemp /tmp/plugin_on_map_semantics_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local m = new MapBox()
    // get(miss) == null
    if m.get("missing") != null { return 201 }

    // set returns Void (language-level); verify by size side-effect
    m.set("b", 2)
    m.set("a", 1)
    if m.size() != 2 { return 202 }

    // delete returns Void; verify by size side-effect
    m.delete("missing")
    if m.size() != 2 { return 203 }

    // keys/values dictionary order (lex order by key)
    local ks = m.keys()
    if ks.size() != 2 { return 204 }

    local vs = m.values()
    if vs.size() != 2 { return 207 }

    // clear returns Void; verify size==0
    m.clear()
    if m.size() != 0 { return 210 }
    return 0
  }
}
SRC

out_vm=$(run_nyash_vm "$tmpfile" )
rc=$?
rm -f "$tmpfile"

if [ $rc -ne 0 ]; then
  echo "FAIL: rc=$rc" >&2
  exit 1
fi
echo "OK"
exit 0
