@echo off
chcp 437 >nul
set "LLVM_SYS_180_PREFIX=C:\LLVM-18"
set "LLVM_SYS_180_FFI_WORKAROUND=1" 
set "LLVM_SYS_180_NO_LIBFFI=1"
set "PATH=C:\LLVM-18\bin;%PATH%"

echo Cleaning and building Nyash with LLVM AOT (no libffi)...
cargo clean
cargo build --bin nyash --release --features llvm

echo.
echo Checking output...
if exist target\release\nyash.exe (
    echo SUCCESS: nyash.exe created
    dir target\release\nyash.exe
) else (
    echo ERROR: nyash.exe not found
)