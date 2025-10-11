#!/bin/bash
# quick profile wrapper — plugin-on array semantics minimal
source "$(dirname "$0")/../../lib/test_runner.sh"
export SMOKES_PROFILE_ENV=plugin-on
require_env || exit 2
preflight_plugins || exit 2
ensure_hako_toml

tmpfile=$(mktemp /tmp/plugin_on_quick_array_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local a = new ArrayBox()
    if a.get(0) != null { return 1 }
    a.set(0, 10)
    if a.size() != 1 { return 2 }
    a.push(20)
    if a.size() != 2 { return 3 }
    if a.get(5) != null { return 4 }
    return 0
  }
}
SRC
run_nyash_vm "$tmpfile" >/dev/null
rc=$?
rm -f "$tmpfile"
if [ $rc -ne 0 ]; then echo "FAIL: rc=$rc"; exit 1; fi
echo "OK"; exit 0
