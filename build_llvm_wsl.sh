#!/bin/bash
set -e

echo "Building Nyash with LLVM for Windows from WSL..."

# Windows側のLLVMを使う
export LLVM_SYS_180_PREFIX="/mnt/c/Program Files/LLVM"

# 追加の環境変数（Qt6ビルドで使っていたかもしれない技）
export LLVM_SYS_180_FFI_WORKAROUND=1
export CC=x86_64-w64-mingw32-gcc
export CXX=x86_64-w64-mingw32-g++
export AR=x86_64-w64-mingw32-ar

# MinGWターゲットで試す（Qt6と同じ方法）
echo "Trying MinGW target..."
cargo build --target x86_64-pc-windows-gnu --release --features llvm

# 成功したら実行ファイルの場所を表示
if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "Binary at: target/x86_64-pc-windows-gnu/release/nyash.exe"
else
    echo "MinGW build failed, trying MSVC target with cargo-xwin..."
    cargo xwin build --target x86_64-pc-windows-msvc --release --features llvm
fi