#!/bin/bash
# selfhost_new_array_size_min_rc_vm.sh — Constructor/Method minimal (rc-only)

source "$(dirname "$0")/../../../lib/test_runner.sh"

require_env || exit 2
ensure_hako_toml

# Prefer builtin core (constructor path) for stability in quick
export NYASH_DISABLE_PLUGINS=1

tmpfile=$(mktemp /tmp/selfhost_new_array_size_min_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main {
  main() {
    local a = new ArrayBox()
    return a.size()  // expect 0
  }
}
SRC

"$NYASH_BIN" --backend vm "$tmpfile" >/dev/null 2> >(filter_noise 1>&2)
rc=$?
rm -f "$tmpfile"
exit $rc

