#!/bin/bash
# plugin_on_string_vm.sh — plugin-on overlay String basic ops

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

ensure_hako_toml

tmpfile=$(mktemp /tmp/plugin_on_string_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local s = "Nyash"
    if s.size() != 5 { return 100 }
    if s.substring(1,3) != "ya" { return 101 }
    if s.indexOf("a") != 2 { return 102 }
    if s.lastIndexOf("h") != 4 { return 103 }
    if s.charAt(0) != "N" { return 104 }
    if ("").isEmpty() != 1 { return 105 }
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

