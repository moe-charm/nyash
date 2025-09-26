#!/bin/bash
set -e

echo "Building Nyash with LLVM for Windows from WSL..."

# Windows cross-compilation: Use legacy inkwell approach for cross-platform builds
LLVM_FEATURE=${NYASH_LLVM_FEATURE:-llvm-inkwell-legacy}
if [[ "$LLVM_FEATURE" == "llvm-inkwell-legacy" ]]; then
  # Windows側のLLVMを使う (legacy inkwell for cross-compilation)
  export LLVM_SYS_180_PREFIX="/mnt/c/Program Files/LLVM"
else
  echo "Warning: Cross-compilation typically requires llvm-inkwell-legacy feature"
  echo "Consider setting NYASH_LLVM_FEATURE=llvm-inkwell-legacy for Windows builds"
fi

# 追加の環境変数（Qt6ビルドで使っていたかもしれない技）
export LLVM_SYS_180_FFI_WORKAROUND=1
export CC=x86_64-w64-mingw32-gcc
export CXX=x86_64-w64-mingw32-g++
export AR=x86_64-w64-mingw32-ar

# MinGWターゲットで試す（Qt6と同じ方法）
echo "Trying MinGW target with feature: $LLVM_FEATURE..."
cargo build --target x86_64-pc-windows-gnu --release --features "$LLVM_FEATURE"

# 成功したら実行ファイルの場所を表示
if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "Binary at: target/x86_64-pc-windows-gnu/release/nyash.exe"
else
    echo "MinGW build failed, trying MSVC target with cargo-xwin..."
    cargo xwin build --target x86_64-pc-windows-msvc --release --features "$LLVM_FEATURE"
fi