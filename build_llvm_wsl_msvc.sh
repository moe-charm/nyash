#!/bin/bash
# Build Windows exe with LLVM from WSL using cross compilation

echo "Setting up Windows cross-compilation with LLVM..."

# Set environment variables for WSL cross-compilation
export LLVM_SYS_180_PREFIX="C:\\LLVM-18"
export LLVM_SYS_180_FFI_WORKAROUND="1"
export LLVM_SYS_NO_LIBFFI="1"  # This is the key!

# Use cargo-xwin for cross compilation
echo "Building nyash.exe for Windows with LLVM support..."
cargo xwin build --target x86_64-pc-windows-msvc --release --features llvm -j32

# Check if successful
if [ -f "target/x86_64-pc-windows-msvc/release/nyash.exe" ]; then
    echo "Build successful!"
    ls -la target/x86_64-pc-windows-msvc/release/nyash.exe
else
    echo "Build failed - nyash.exe not found"
    echo "Checking what was built:"
    ls -la target/x86_64-pc-windows-msvc/release/ 2>/dev/null || echo "Target directory not found"
fi