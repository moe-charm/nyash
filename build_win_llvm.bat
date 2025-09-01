@echo off
echo Building Nyash with LLVM for Windows...

REM Set environment variables
set LLVM_SYS_180_PREFIX=C:\LLVM-18
set LLVM_SYS_180_FFI_WORKAROUND=1
set LLVM_SYS_NO_LIBFFI=1

echo Environment variables:
echo LLVM_SYS_180_PREFIX = %LLVM_SYS_180_PREFIX%
echo LLVM_SYS_180_FFI_WORKAROUND = %LLVM_SYS_180_FFI_WORKAROUND%
echo LLVM_SYS_NO_LIBFFI = %LLVM_SYS_NO_LIBFFI%

echo.
echo Building...
cargo build --release --features llvm

echo.
echo Done!
pause