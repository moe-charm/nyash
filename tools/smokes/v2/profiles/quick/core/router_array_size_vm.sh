#!/usr/bin/env bash
set -euo pipefail

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

export NYASH_USE_CALL_ROUTER=1
export NYASH_CALL_ROUTER_TRACE=${NYASH_CALL_ROUTER_TRACE:-0}

PROG=$(mktemp)
cat >"$PROG" <<'NYASH'
static box Main {
  main() {
    local arr = new ArrayBox()
    arr.push(1)
    local len1 = arr.length()
    arr.push(2)
    local len2 = arr.size()
    print(len1)
    print(len2)
  }
}
NYASH

OUT=$(run_nyash_vm "$PROG")

L1=$(echo "$OUT" | sed -n '1p')
L2=$(echo "$OUT" | sed -n '2p')

[[ "$L1" =~ ^[0-9]+$ ]] || fail "len1 not numeric: $L1"
[[ "$L2" =~ ^[0-9]+$ ]] || fail "len2 not numeric: $L2"

if [[ "$L1" -ne 1 ]]; then
  fail "unexpected len1: $L1"
fi

if [[ "$L2" -ne 2 ]]; then
  fail "unexpected len2: $L2"
fi

pass
