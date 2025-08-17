#!/bin/bash
# FileBox パフォーマンス比較

echo "🔬 FileBox 静的版 vs 動的版 パフォーマンス比較"
echo "============================================="
echo "テスト内容: 100個のファイル作成・書き込み・読み込み・存在確認"
echo ""

# クリーンアップ
rm -f perf_test_*.txt

# 動的版（現在）
echo "1️⃣ 動的版 (dynamic-file feature 有効)"
echo -n "   実行時間: "
{ time RUST_LOG=error ./target/release/nyash local_tests/benchmark_filebox_simple.nyash > /dev/null 2>&1; } 2>&1 | grep real | awk '{print $2}'
rm -f perf_test_*.txt

# 静的版ビルド
echo ""
echo "   静的版をビルド中..."
cargo build --release --no-default-features -j32 > /dev/null 2>&1

# 静的版
echo ""
echo "2️⃣ 静的版 (FileBox組み込み)"
echo -n "   実行時間: "
{ time RUST_LOG=error ./target/release/nyash local_tests/benchmark_filebox_simple.nyash > /dev/null 2>&1; } 2>&1 | grep real | awk '{print $2}'
rm -f perf_test_*.txt

# 複数回測定
echo ""
echo "📊 5回測定の平均:"
echo ""
echo "動的版:"
for i in {1..5}; do
    echo -n "  Run $i: "
    { time RUST_LOG=error ./target/release/nyash local_tests/benchmark_filebox_simple.nyash > /dev/null 2>&1; } 2>&1 | grep real | awk '{print $2}'
    rm -f perf_test_*.txt
done

# 静的版に切り替え
cargo build --release --no-default-features -j32 > /dev/null 2>&1

echo ""
echo "静的版:"
for i in {1..5}; do
    echo -n "  Run $i: "
    { time RUST_LOG=error ./target/release/nyash local_tests/benchmark_filebox_simple.nyash > /dev/null 2>&1; } 2>&1 | grep real | awk '{print $2}'
    rm -f perf_test_*.txt
done

# 動的版に戻す
echo ""
echo "元の動的版に戻しています..."
cargo build --release -j32 > /dev/null 2>&1

echo ""
echo "✅ 測定完了！"