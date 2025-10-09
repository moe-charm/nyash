#!/bin/bash
# plugin_on_basic_vm.sh — plugin-on overlay basic VM sanity

source "$(dirname "$0")/../../../lib/test_runner.sh"
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

ensure_hako_toml

tmpfile=$(mktemp /tmp/plugin_on_basic_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main { main() { return 42 } }
SRC

out_vm=$(run_nyash_vm "$tmpfile" | awk '/^Result:/{print $0}' | head -n1 | tr -d '\r' | xargs)
rm -f "$tmpfile"

if [ "$out_vm" != "Result: 42" ]; then
  echo "FAIL: expected 'Result: 42', got '$out_vm'" >&2
  exit 1
fi
echo "$out_vm"
exit 0

