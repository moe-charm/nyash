#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)
source "$ROOT/tools/test/lib/shlib.sh"

build_nyash_release

export NYASH_LLVM_USE_HARNESS=1
# PHI-off + if-merge prepass enabled
export NYASH_MIR_NO_PHI=${NYASH_MIR_NO_PHI:-1}
export NYASH_VERIFY_ALLOW_NO_PHI=${NYASH_VERIFY_ALLOW_NO_PHI:-1}
export NYASH_LLVM_PREPASS_IFMERGE=1

APP="$ROOT/apps/tests/ternary_basic.nyash"
# Expect exit code (default 0); allow override via NYASH_LLVM_EXPECT_EXIT
EXPECT=${NYASH_LLVM_EXPECT_EXIT:-0}
assert_exit "timeout -s KILL 20s $ROOT/target/release/nyash --backend llvm $APP >/dev/null" "$EXPECT"
echo "OK: llvm if-merge (ternary_basic exit=$EXPECT)"
