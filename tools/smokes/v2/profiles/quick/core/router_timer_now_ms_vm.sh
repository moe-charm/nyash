#!/usr/bin/env bash

# Router を有効化して TimerBox.now_ms の直行Extern経路と単調増加を確認

DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
. "${DIR}/../../../lib/test_runner.sh"

require_env || exit 2
preflight_plugins || exit 2

# Router をON（トレース任意） + コア常在ルール（プラグイン無効）
export NYASH_USE_CALL_ROUTER=1
export NYASH_CALL_ROUTER_TRACE=${NYASH_CALL_ROUTER_TRACE:-0}
export NYASH_DISABLE_PLUGINS=1
# TimerBox はコア解決を想定するが、環境差吸収のため明示モジュールも併用
export NYASH_MODULES=selfhost.core.timer

PROG=$(mktemp)
cat >"$PROG" <<'NYASH'
// コア常在ルール: new/using なし、静的呼び出しで確認
static box Main {
  main() {
    // 3 回取得して単調非減を確認（同値許容）。
    local v1 = TimerBox.now_ms()
    local v2 = TimerBox.now_ms()
    if (v2 < v1) {
      print("ng")
      return 0
    }
    local v3 = TimerBox.now_ms()
    if (v3 < v2) {
      print("ng")
      return 0
    }
    print("ok")
    return 0
  }
}
NYASH

ensure_hako_toml
OUT=$(run_nyash_vm "$PROG")

if [[ "$OUT" != "ok" ]]; then
  fail "timer monotonic check failed: $OUT"
fi

pass "timer monotonic: ok"
