#!/bin/bash
# plugin_on_string_replace_edges_vm.sh — plugin-on overlay String replace edges

source "$(dirname "$0")/../../../lib/test_runner.sh"
export NYASH_DISABLE_PLUGINS=0
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

ensure_hako_toml

tmpfile=$(mktemp /tmp/plugin_on_string_replace_edges_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local s1 = "banana"
    if s1.indexOf("na") != 2 { return 201 }
    if s1.indexOf("x") != -1 { return 202 }
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
