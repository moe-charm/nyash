#!/bin/bash
# JIT (Cranelift) ビルド - 24スレッド並列
echo "🚀 JIT (Cranelift) ビルドを開始します..."
cargo build --release --features cranelift-jit -j 24
echo "✅ JIT ビルド完了！"