#!/bin/bash
# plugin_off_string_boxcall_vm.sh — StringBox BoxCall sanity with plugins OFF

source "$(dirname "$0")/../../../lib/test_runner.sh"
if [ "${SMOKES_SELFHOST_ENABLE:-0}" != "1" ]; then test_skip "selfhost suite gated (set SMOKES_SELFHOST_ENABLE=1)"; exit 0; fi
export HAKO_PLUGIN_POLICY=off
export NYASH_DISABLE_PLUGINS=1
require_env || exit 2
preflight_plugins || exit 2

tmpfile=$(mktemp /tmp/plugin_off_string_boxcall_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local s = "PluginOff"
    if s.indexOf("Off") != 6 { return 101 }
    if s.substring(0, 6) != "Plugin" { return 102 }
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

