#!/bin/bash
# LLVM ビルド - 24スレッド並列（llvmliteハーネス）
echo "🚀 LLVM ビルド開始（LLVM_SYS_180_PREFIX不要）..."
cargo build --release --features llvm -j 24
echo "✅ LLVM ビルド完了！"