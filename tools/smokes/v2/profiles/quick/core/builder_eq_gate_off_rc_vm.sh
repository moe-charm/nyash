#!/bin/bash
# builder_eq_gate_off_rc_vm.sh — Gate OFF: NYASH_BUILDER_EQ_TO_OPEQ=0 (Eq stays Compare), rc-only

source "$(dirname "$0")/../../../lib/test_runner.sh"

export NYASH_BUILDER_EQ_TO_OPEQ=0
require_env || exit 2
ensure_hako_toml

tmpfile=$(mktemp /tmp/builder_eq_gate_off_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main { main() { if (7 == 7) { return 0 } else { return 1 } } }
SRC

"$NYASH_BIN" --backend vm "$tmpfile" >/dev/null 2> >(filter_noise 1>&2)
rc=$?
rm -f "$tmpfile"
exit $rc

