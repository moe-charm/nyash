#!/bin/bash
# quick profile wrapper — plugin-on-strict map semantics minimal
source "$(dirname "$0")/../../lib/test_runner.sh"
export SMOKES_PROFILE_ENV=plugin-on-strict
require_env || exit 2
preflight_plugins || exit 2
ensure_hako_toml

tmpfile=$(mktemp /tmp/strict_quick_map_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local m = new MapBox()
    if m.get("x") != null { return 1 }
    m.set("b", 2)
    m.set("a", 1)
    local ks = m.keys()
    if ks.get(0) != "a" || ks.get(1) != "b" { return 2 }
    if m.get(ks.get(0)) != 1 || m.get(ks.get(1)) != 2 { return 3 }
    return 0
  }
}
SRC
out=$(run_nyash_vm "$tmpfile" | awk '/^Result:/{print $0}' | head -n1 | tr -d '
' | xargs)
rm -f "$tmpfile"
if [ "$out" != "Result: 0" ]; then
  echo "FAIL: expected 'Result: 0', got '$out'" >&2
  exit 1
fi
echo "$out"
exit 0
