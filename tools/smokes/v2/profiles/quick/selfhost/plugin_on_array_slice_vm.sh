#!/bin/bash
# plugin_on_array_slice_vm.sh — plugin-on overlay Array set/get/len (boundary)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_DISABLE_PLUGIN_CHECKS=${SMOKES_DISABLE_PLUGIN_CHECKS:-1}
export NYASH_DISABLE_PLUGINS=0
export SMOKES_USE_PYVM=0
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

tmpfile=$(mktemp /tmp/plugin_on_array_slice_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local a = new ArrayBox()
    a.push(1)
    // set at end appends
    a.set(1, 7)
    if a.size() != 2 { return 201 }
    // get within bounds is not null; out-of-bounds is null
    if a.get(1) == null { return 202 }
    if a.get(5) != null { return 203 }
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
