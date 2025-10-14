#!/bin/bash
# Build Windows exe with LLVM from WSL using cross compilation

echo "Setting up Windows cross-compilation with LLVM..."

# Windows cross-compilation: Use legacy inkwell approach for cross-platform builds
LLVM_FEATURE=${NYASH_LLVM_FEATURE:-llvm-inkwell-legacy}
if [[ "$LLVM_FEATURE" == "llvm-inkwell-legacy" ]]; then
  # Set environment variables for WSL cross-compilation (legacy inkwell)
  export LLVM_SYS_180_PREFIX="C:\\LLVM-18"
  export LLVM_SYS_180_FFI_WORKAROUND="1"
  export LLVM_SYS_NO_LIBFFI="1"  # This is the key!
else
  echo "Warning: Cross-compilation typically requires llvm-inkwell-legacy feature"
  echo "Consider setting NYASH_LLVM_FEATURE=llvm-inkwell-legacy for Windows builds"
fi

# Use cargo-xwin for cross compilation
echo "Building nyash.exe for Windows with LLVM support (feature: $LLVM_FEATURE)..."
cargo xwin build --target x86_64-pc-windows-msvc --release --features "$LLVM_FEATURE" -j32

# Check if successful
if [ -f "target/x86_64-pc-windows-msvc/release/nyash.exe" ]; then
    echo "Build successful!"
    ls -la target/x86_64-pc-windows-msvc/release/nyash.exe
else
    echo "Build failed - nyash.exe not found"
    echo "Checking what was built:"
    ls -la target/x86_64-pc-windows-msvc/release/ 2>/dev/null || echo "Target directory not found"
fi