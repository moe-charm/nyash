#!/bin/bash
# plugin_on_map_values_identity_vm.sh — identity round‑trip: Map stores Array handle, get returns same instance

source "$(dirname "$0")/../../../lib/test_runner.sh"
export NYASH_DISABLE_PLUGINS=0
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

ensure_hako_toml

tmpfile=$(mktemp /tmp/plugin_on_map_values_identity_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    // Create Array and mutate
    local a = new ArrayBox()
    a.push(1)
    // Put into Map under key "x" (as handle)
    local m = new MapBox()
    m.set("x", a)
    // Get back
    local a2 = m.get("x")
    if a2 == null { return 201 }
    // Mutate original and observe via alias
    a.push(2)
    if a2.size() != a.size() { return 202 }
    // Mutate alias and observe original
    a2.push(3)
    if a.size() != 3 { return 203 }
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
