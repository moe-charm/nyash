#!/bin/bash
# Build Windows MSVC test executable
# Phase 15.77 - MSVC Static Runtime Route
# Usage: bash tools/aot/windows/build_test_msvc.sh

set -e

echo "=== Phase 15.77: MSVC Static Runtime Build ==="
echo ""

# Step 1: Build hako_kernel.lib (MSVC target)
echo "[1/3] Building hako_kernel.lib (x86_64-pc-windows-msvc)..."
cargo build -p hako_kernel --release \
    --target x86_64-pc-windows-msvc \
    --no-default-features -F core-runtime

KERNEL_LIB="target/x86_64-pc-windows-msvc/release/hako_kernel.lib"
if [ ! -f "$KERNEL_LIB" ]; then
    echo "Error: $KERNEL_LIB not found"
    exit 1
fi
echo "✓ hako_kernel.lib generated ($(du -h $KERNEL_LIB | cut -f1))"
echo ""

# Step 2: Compile ny_main_win.c to COFF .obj
echo "[2/3] Compiling ny_main_win.c with Windows clang.exe..."
NY_MAIN_C="tools/aot/windows/ny_main_win.c"
if [ ! -f "$NY_MAIN_C" ]; then
    echo "Error: $NY_MAIN_C not found"
    exit 1
fi

# Copy files to Windows temp
mkdir -p /mnt/c/temp
cp "$KERNEL_LIB" /mnt/c/temp/hako_kernel.lib
cp "$NY_MAIN_C" /mnt/c/temp/ny_main_win.c

# Compile with Windows clang.exe
/mnt/c/LLVM-18/bin/clang.exe \
    --target=x86_64-pc-windows-msvc \
    -c C:\\temp\\ny_main_win.c \
    -o C:\\temp\\ny_main_win.obj

if [ ! -f /mnt/c/temp/ny_main_win.obj ]; then
    echo "Error: ny_main_win.obj not generated"
    exit 1
fi
echo "✓ ny_main_win.obj generated ($(du -h /mnt/c/temp/ny_main_win.obj | cut -f1))"
echo ""

# Step 3: Link with hako_kernel.lib
echo "[3/3] Linking test_msvc.exe..."
/mnt/c/LLVM-18/bin/clang.exe \
    --target=x86_64-pc-windows-msvc \
    C:\\temp\\ny_main_win.obj \
    C:\\temp\\hako_kernel.lib \
    -lws2_32 -ladvapi32 -luserenv -lole32 -lbcrypt -lntdll \
    -luser32 -lkernel32 -lshell32 \
    -Wl,/FORCE:MULTIPLE \
    -o C:\\temp\\test_msvc.exe 2>&1 | grep -v "LNK4006\|LNK4088" || true

if [ ! -f /mnt/c/temp/test_msvc.exe ]; then
    echo "Error: test_msvc.exe not generated"
    exit 1
fi
echo "✓ test_msvc.exe generated ($(du -h /mnt/c/temp/test_msvc.exe | cut -f1))"
echo ""

# Step 4: Execute and verify
echo "[4/4] Executing test_msvc.exe..."
OUTPUT=$(/mnt/c/temp/test_msvc.exe 2>&1)
echo "$OUTPUT"
echo ""

if echo "$OUTPUT" | grep -q "Result: 6"; then
    echo "✅ SUCCESS: MSVC route complete!"
    echo ""
    echo "Output file: C:\\temp\\test_msvc.exe"
    echo "Size: $(du -h /mnt/c/temp/test_msvc.exe | cut -f1)"
    exit 0
else
    echo "❌ FAILED: Expected 'Result: 6', got: $OUTPUT"
    exit 1
fi
