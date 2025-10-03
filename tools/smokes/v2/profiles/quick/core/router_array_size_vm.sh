#!/usr/bin/env bash

source "$(dirname "$0")/../../../lib/test_runner.sh"
require_env || exit 2
preflight_plugins || exit 2

export NYASH_USE_CALL_ROUTER=1
export NYASH_CALL_ROUTER_TRACE=${NYASH_CALL_ROUTER_TRACE:-0}
# コア常在ルール: プラグインは無効化（ビルトイン実装で動作）
export NYASH_DISABLE_PLUGINS=1

PROG=$(mktemp)
# 事前プローブ: 空配列の size() が数値を返せるか（Router/Adapter経路の可用性）
PROBE=$(run_nyash_vm -c 'static box Main { main() { local a = new ArrayBox() local s = a.size() print(s) return 0 } }' --dev 2>/dev/null | tail -n 1 | tr -d '\r' | xargs || true)
if ! [[ "$PROBE" =~ ^[0-9]+$ ]]; then
  test_skip "router_array_size_vm" "Router not available (probe='$PROBE')"
  exit 0
fi
cat >"$PROG" <<'NYASH'
static box Main {
  main() {
    local arr = new ArrayBox()
    arr.push(1)
    local len1 = arr.length()
    arr.push(2)
    local len2 = arr.size()
    if (len1 == 1 && len2 == 2) {
      print("ok")
    } else {
      print("ng")
    }
  }
}
NYASH

OUT=$(run_nyash_vm "$PROG" 2>&1 || true)
[[ "$OUT" == "ok" ]] || fail "array size route failed: $OUT"
pass "array size route: ok"
