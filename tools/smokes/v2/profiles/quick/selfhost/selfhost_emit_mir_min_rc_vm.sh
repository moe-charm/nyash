#!/bin/bash
# selfhost_emit_mir_min_rc_vm.sh — quick 常時: 自己ホスト emit-mir 最小（rc-only）

source "$(dirname "$0")/../../../lib/test_runner.sh"

require_env || exit 2
ensure_hako_toml

# Use selfhost compiler child to emit MIR(JSON) then execute via Rust VM.
export NYASH_USE_NY_COMPILER=1
export NYASH_NY_COMPILER_MIN_JSON=1
export NYASH_NY_COMPILER_CHILD_ARGS="--emit-mir"
export NYASH_JSON_ONLY=1

"$NYASH_BIN" --backend vm "$NYASH_ROOT/apps/examples/string_p0.hako" >/dev/null 2> >(filter_noise 1>&2)
exit $?

