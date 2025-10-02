#!/usr/bin/env bash
set -euo pipefail

# Router を有効化して TimerBox.now_ms の直行Extern経路と単調増加を確認

DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
. "${DIR}/../../lib/test_runner.sh"

require_nyash_vm || skip "VM runner unavailable"

# Router をON（トレース任意）
export NYASH_USE_CALL_ROUTER=1
export NYASH_CALL_ROUTER_TRACE=${NYASH_CALL_ROUTER_TRACE:-0}

PROG=$(mktemp)
cat >"$PROG" <<'NYASH'
flow Main {
  static main() {
    // Var 受け手
    local t = new TimerBox()
    local v1 = t.now_ms()
    // Me 受け手（静的Box内のインスタンス相当は省略、Var/Static/Fieldを網羅）
    // Static 受け手
    local v2 = TimerBox.now_ms()
    // FieldAccess 受け手: obj.field.now_ms() 形式を模すため、Boxをそのままフィールドに入れて呼ぶ
    local holder = new MapBox()
    holder.set("timer", t)
    local t2 = holder.get("timer")
    local v3 = t2.now_ms()
    print(v1)
    print(v2)
    print(v3)
  }
}
NYASH

OUT=$(run_nyash_vm "$PROG")

# 出力3行の数値が単調非減（通常は増加）であり、少なくとも v1 != v2 の期待（CSE/折り畳み抑止）
V1=$(echo "$OUT" | sed -n '1p')
V2=$(echo "$OUT" | sed -n '2p')
V3=$(echo "$OUT" | sed -n '3p')

[[ "$V1" =~ ^[0-9]+$ ]] || fail "v1 is not number: $V1"
[[ "$V2" =~ ^[0-9]+$ ]] || fail "v2 is not number: $V2"
[[ "$V3" =~ ^[0-9]+$ ]] || fail "v3 is not number: $V3"

if [[ "$V1" -eq "$V2" ]]; then
  fail "Router ON but v1==v2 (CSE or routing issue): $V1 == $V2"
fi

if [[ "$V2" -gt "$V3" ]]; then
  fail "non‑monotonic: v2=$V2 v3=$V3"
fi

pass

