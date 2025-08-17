#!/bin/bash
# パフォーマンス測定スクリプト

echo "🔬 FileBox パフォーマンス測定"
echo "================================"

# クリーンアップ
rm -f perf_test_*.txt benchmark_test*.txt

# 動的版の測定
echo -e "\n📊 動的版 (dynamic-file feature 有効)"
echo "開始時刻: $(date +%H:%M:%S.%N)"
START=$(date +%s%N)

RUST_LOG=error ./target/release/nyash local_tests/benchmark_filebox_simple.nyash 2>/dev/null

END=$(date +%s%N)
ELAPSED=$((($END - $START) / 1000000))
echo "終了時刻: $(date +%H:%M:%S.%N)"
echo "⏱️  実行時間: ${ELAPSED}ms"

# クリーンアップ
rm -f perf_test_*.txt

# 静的版のビルド
echo -e "\n🔧 静的版をビルド中..."
cargo build --release --no-default-features 2>/dev/null

# 静的版の測定  
echo -e "\n📊 静的版 (dynamic-file feature 無効)"
echo "開始時刻: $(date +%H:%M:%S.%N)"
START=$(date +%s%N)

RUST_LOG=error ./target/release/nyash local_tests/benchmark_filebox_simple.nyash 2>/dev/null

END=$(date +%s%N)
ELAPSED=$((($END - $START) / 1000000))
echo "終了時刻: $(date +%H:%M:%S.%N)"
echo "⏱️  実行時間: ${ELAPSED}ms"

# クリーンアップ
rm -f perf_test_*.txt

echo -e "\n✅ 測定完了！"