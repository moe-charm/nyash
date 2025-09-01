@echo off
echo Setting system environment variables for LLVM...

REM Set system-wide environment variables
setx LLVM_SYS_180_PREFIX "C:\LLVM-18"
setx LLVM_SYS_NO_LIBFFI "1"
setx LLVM_SYS_180_FFI_WORKAROUND "1"

echo.
echo Environment variables set. Please open a NEW command prompt and run:
echo cargo build --bin nyash --release --features llvm
echo.
pause