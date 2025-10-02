#!/usr/bin/env bash

# Router を有効化して TimerBox.now_ms の直行Extern経路と単調増加を確認

DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
. "${DIR}/../../../lib/test_runner.sh"

require_env || exit 2
preflight_plugins || exit 2

# Router をON（トレース任意）
export NYASH_USE_CALL_ROUTER=1
export NYASH_CALL_ROUTER_TRACE=${NYASH_CALL_ROUTER_TRACE:-0}
export NYASH_MODULES=selfhost.core.timer

PROG=$(mktemp)
cat >"$PROG" <<'NYASH'
using selfhost.core.timer as TimerBox

static box Main {
  main() {
    // Var 受け手
    local t = new TimerBox()
    local v1 = t.now_ms()
    // Static 受け手
    local v2 = TimerBox.now_ms()
    // 再度 Var 受け手（CSE抑止と単調性確認）
    local v3 = t.now_ms()
    print(v1)
    print(v2)
    print(v3)
  }
}
NYASH

ensure_hako_toml
OUT=$(run_nyash_vm "$PROG")

# 出力3行の数値が単調非減（ミリ秒解像度のため同値も許容）
V1=$(echo "$OUT" | sed -n '1p')
V2=$(echo "$OUT" | sed -n '2p')
V3=$(echo "$OUT" | sed -n '3p')

[[ "$V1" =~ ^[0-9]+$ ]] || fail "v1 is not number: $V1"
[[ "$V2" =~ ^[0-9]+$ ]] || fail "v2 is not number: $V2"
[[ "$V3" =~ ^[0-9]+$ ]] || fail "v3 is not number: $V3"

if [[ "$V2" -gt "$V3" ]]; then
  fail "non‑monotonic: v2=$V2 v3=$V3"
fi

pass
