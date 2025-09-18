#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)
source "$ROOT/tools/test/lib/shlib.sh"

build_nyash_release

export NYASH_LLVM_USE_HARNESS=1
export NYASH_MIR_NO_PHI=${NYASH_MIR_NO_PHI:-1}
export NYASH_VERIFY_ALLOW_NO_PHI=${NYASH_VERIFY_ALLOW_NO_PHI:-1}

APP="$ROOT/apps/tests/loop_if_phi.nyash"
assert_exit "timeout -s KILL 20s $ROOT/target/release/nyash --backend llvm $APP >/dev/null" 0
echo "OK: llvm quick (loop_if_phi)"

