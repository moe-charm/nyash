#!/bin/bash
# LLVM ビルド - 24スレッド並列
echo "🚀 LLVM ビルドを開始します..."
export LLVM_SYS_180_PREFIX=/usr/lib/llvm-18
cargo build --release --features llvm -j 24
echo "✅ LLVM ビルド完了！"