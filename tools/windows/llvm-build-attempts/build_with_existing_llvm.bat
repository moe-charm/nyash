@echo off
echo Using existing LLVM at C:\LLVM-18

REM Set environment variables
set "LLVM_SYS_180_PREFIX=C:\LLVM-18"
set "LLVM_SYS_180_NO_LIBFFI=1"
set "LLVM_SYS_180_FFI_WORKAROUND=1"
set "PATH=C:\LLVM-18\bin;%PATH%"

echo.
echo Building Nyash without libffi (AOT only)...
cargo build --bin nyash --release --features llvm --no-default-features --features cli,plugins

echo.
if exist target\release\nyash.exe (
    echo SUCCESS: nyash.exe created!
    dir target\release\nyash.exe
) else (
    echo Build failed. Trying alternative approach...
)