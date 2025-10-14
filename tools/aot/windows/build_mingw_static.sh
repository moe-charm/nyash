#!/usr/bin/env bash
set -euo pipefail

# Quicklink: Build and link a tiny ny_main against libhako_kernel.a (MinGW)
# Prereqs:
#   - x86_64-w64-mingw32-gcc in PATH (WSL OK)
#   - Rust-built static runtime: target/x86_64-pc-windows-gnu/release/libhako_kernel.a

KERNEL_A=${1:-target/x86_64-pc-windows-gnu/release/libhako_kernel.a}
OUT_EXE=${2:-build/test_min.exe}

mkdir -p build

echo "[1/2] Compiling ny_main (x86_64-w64-mingw32-gcc)" >&2
x86_64-w64-mingw32-gcc -c tools/aot/windows/ny_main_win.c -o build/ny_main_win.o

echo "[2/2] Linking $OUT_EXE" >&2
x86_64-w64-mingw32-gcc -static build/ny_main_win.o \
  "$KERNEL_A" \
  -Wl,--allow-multiple-definition \
  -lws2_32 -ladvapi32 -luserenv -lole32 -lbcrypt -lntdll -luser32 -lkernel32 \
  -o "$OUT_EXE"

echo "OK: $OUT_EXE" >&2

