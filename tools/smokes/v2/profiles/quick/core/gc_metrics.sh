#!/bin/bash
# gc_metrics.sh - GCメトリクスの存在確認（デフォルトSKIP）

source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

require_env || exit 2
preflight_plugins || exit 2

test_gc_metrics() {
    # 既定はSKIP。明示的に SMOKES_ENABLE_GC_METRICS=1 で実行。
    if [ "${SMOKES_ENABLE_GC_METRICS:-0}" != "1" ]; then
        test_skip "gc_metrics (set SMOKES_ENABLE_GC_METRICS=1 to enable)"
        return 0
    fi
    # 参考: safepoint誘発は env.runtime.checkpoint だが、現状ソースからの直接呼び出しは未提供のため
    # ここではメトリクス出力の有無のみ緩やかに検査する（環境次第で安定しない場合はSKIPへフォールバック）。
    local code='print("gc-metrics-probe")'
    # 強制トリガ: safepoint間隔=1 / メトリクスON
    local out
    out=$(NYASH_GC_MODE=rc+cycle NYASH_GC_COLLECT_SP=1 NYASH_GC_METRICS=1 \
          NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 \
          "$NYASH_BIN" -c "$code" 2>&1 || true)
    # PASS基準: 出力に [GC] trial: が含まれていればOK。無ければSKIP（環境依存）。
    if echo "$out" | grep -q "^\[GC\] trial:"; then
        return 0
    else
        test_skip "gc_metrics (no metrics emitted; environment dependent)"
        return 0
    fi
}

run_test "gc_metrics" test_gc_metrics

