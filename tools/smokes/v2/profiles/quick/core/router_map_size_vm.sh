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
    local m = new MapBox()
    m.set("a", 1)
    local s1 = m.size()
    m.set("b", 2)
    local s2 = m.size()
    print(s1)
    print(s2)
  }
}
NYASH

OUT=$(run_nyash_vm "$PROG")

S1=$(echo "$OUT" | sed -n '1p')
S2=$(echo "$OUT" | sed -n '2p')

[[ "$S1" =~ ^[0-9]+$ ]] || fail "size1 not numeric: $S1"
[[ "$S2" =~ ^[0-9]+$ ]] || fail "size2 not numeric: $S2"

if [[ "$S1" -ne 1 ]]; then
  fail "unexpected map size1: $S1"
fi

if [[ "$S2" -ne 2 ]]; then
  fail "unexpected map size2: $S2"
fi

pass
