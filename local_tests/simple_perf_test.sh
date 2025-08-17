#!/bin/bash
# シンプルなパフォーマンス測定

echo "🔬 FileBox パフォーマンス測定（100回のファイル操作）"
echo "=================================================="

# クリーンアップ
rm -f perf_test_*.txt

# 動的版
echo -e "\n📊 動的版 (現在のビルド)"
echo "計測開始..."
time RUST_LOG=error ./target/release/nyash local_tests/benchmark_filebox_simple.nyash > /dev/null 2>&1
rm -f perf_test_*.txt

# 静的版ビルド
echo -e "\n🔧 静的版をビルド中..."
cargo build --release --no-default-features -j32 > /dev/null 2>&1

# 静的版
echo -e "\n📊 静的版 (dynamic-file無効)"  
echo "計測開始..."
time RUST_LOG=error ./target/release/nyash local_tests/benchmark_filebox_simple.nyash > /dev/null 2>&1
rm -f perf_test_*.txt

# 動的版に戻す
echo -e "\n🔧 動的版に戻します..."
cargo build --release -j32 > /dev/null 2>&1

echo -e "\n✅ 測定完了！"