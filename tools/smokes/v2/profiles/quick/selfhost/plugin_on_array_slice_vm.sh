#!/bin/bash
# plugin_on_array_slice_vm.sh — plugin-on overlay Array set/get/len (boundary)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export NYASH_DISABLE_PLUGINS=0
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

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

out_vm=$(run_nyash_vm "$tmpfile" | awk '/^Result:/{print $0}' | head -n1 | tr -d '\r' | xargs)
rm -f "$tmpfile"

if [ "$out_vm" != "Result: 0" ]; then
  echo "FAIL: expected 'Result: 0', got '$out_vm'" >&2
  exit 1
fi
echo "$out_vm"
exit 0
