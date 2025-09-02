@echo off
setlocal

REM Set LLVM path
set "LLVM_SYS_180_PREFIX=C:\Program Files\LLVM"

REM Build nyash with LLVM feature
echo Building nyash with LLVM backend...
cargo build --release --features llvm

REM Build ny-echo-lite to exe
echo Building ny-echo-lite...
powershell -ExecutionPolicy Bypass -File tools\build_llvm.ps1 apps\tests\ny-echo-lite\main.nyash -Out app_echo.exe

echo Done!
endlocal