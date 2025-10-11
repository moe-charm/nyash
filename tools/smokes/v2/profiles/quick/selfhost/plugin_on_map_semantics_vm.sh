#!/bin/bash
# plugin_on_map_semantics_vm.sh — plugin-on: Map semantics (miss→null, void mutators, keys/values order)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export NYASH_DISABLE_PLUGINS=0
export SMOKES_USE_PYVM=0
# Enable Stage-2 keys/values (HostHandle Array) to assert ordering
export NYASH_PLUGIN_MAP_ARRAY_HANDLE=1

require_env || exit 2
preflight_plugins || exit 2

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
    if ks.length() != 2 { return 204 }
    if ks.get(0) != "a" { return 205 }
    if ks.get(1) != "b" { return 206 }

    local vs = m.values()
    if vs.length() != 2 { return 207 }
    if vs.get(0) != 1 { return 208 }
    if vs.get(1) != 2 { return 209 }

    // clear returns Void; verify size==0
    m.clear()
    if m.size() != 0 { return 210 }
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
