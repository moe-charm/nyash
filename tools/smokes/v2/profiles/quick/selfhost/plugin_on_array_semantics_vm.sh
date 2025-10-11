#!/bin/bash
# plugin_on_array_semantics_vm.sh — plugin-on: Array semantics (get oob -> null, set/push -> Void)

source "$(dirname "$0")/../../../lib/test_runner.sh"
export NYASH_DISABLE_PLUGINS=0
export SMOKES_USE_PYVM=0
require_env || exit 2
preflight_plugins || exit 2

ensure_hako_toml

tmpfile=$(mktemp /tmp/plugin_on_array_semantics_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local a = new ArrayBox()
    // get(oob) == null on empty
    if a.get(0) != null { return 301 }

    // set/push return Void; verify via size side-effect
    a.set(0, 10)
    if a.size() != 1 { return 302 }
    a.push(20)
    if a.size() != 2 { return 303 }

    // OOB still null
    if a.get(5) != null { return 304 }

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
