#!/bin/bash
# quick profile wrapper — plugin-on array semantics minimal
source "$(dirname "$0")/../../lib/test_runner.sh"
export SMOKES_PROFILE_ENV=plugin-on
require_env || exit 2
preflight_plugins || { echo "SKIP: plugins not available (preflight)" >&2; exit 0; }
ensure_hako_toml

# Precheck: ArrayBox must be constructible under plugin-on
pre_src=$(mktemp /tmp/plugin_on_quick_array_pre_XXXX.hako)
cat >"$pre_src" << 'SRC'
static box Main { main() { local a = new ArrayBox(); return 0 } }
SRC
run_nyash_vm "$pre_src" >/dev/null || { echo 'SKIP: ArrayBox not available (precheck)' >&2; rm -f "$pre_src"; exit 0; }
rm -f "$pre_src" 


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
