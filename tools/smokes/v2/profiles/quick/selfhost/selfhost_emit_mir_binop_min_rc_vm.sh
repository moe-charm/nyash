#!/bin/bash
# selfhost_emit_mir_binop_min_rc_vm.sh — quick 常時: 自己ホスト emit-mir binop 最小（rc-only）

source "$(dirname "$0")/../../../lib/test_runner.sh"

require_env || exit 2
ensure_hako_toml

export NYASH_USE_NY_COMPILER=1
export NYASH_NY_COMPILER_MIN_JSON=1
export NYASH_NY_COMPILER_CHILD_ARGS="--emit-mir"
export NYASH_JSON_ONLY=1

# Minimal driver: (+) 3 and 4 via compiler path; rc-only
tmpfile=$(mktemp /tmp/selfhost_emit_mir_binop_min_XXXX.hako)
cat >"$tmpfile" << 'SRC'
static box Main { main() { return 3 + 4 } }
SRC

"$NYASH_BIN" --backend vm "$tmpfile" >/dev/null 2> >(filter_noise 1>&2)
rc=$?
rm -f "$tmpfile"
exit $rc

