#!/bin/bash
# plugin_on_string_search_edges_vm.sh — plugin-on overlay String find/lastIndexOf edges

source "$(dirname "$0")/../../../lib/test_runner.sh"
export NYASH_DISABLE_PLUGINS=0
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

ensure_hako_toml

tmpfile=$(mktemp /tmp/plugin_on_string_search_edges_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local s = "Hello, Nyash!"
    if s.indexOf("Ny") != 7 { return 101 }
    if s.indexOf("xyz") != -1 { return 102 }
    if s.lastIndexOf("l") != 3 { return 103 } // H e l l o → last 'l' at index 3
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
