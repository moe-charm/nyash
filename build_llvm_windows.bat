@echo off
setlocal enabledelayedexpansion

echo [build_llvm_windows] Starting LLVM build...

REM Set LLVM path
set "LLVM_SYS_180_PREFIX=C:\Program Files\LLVM"
set "PATH=C:\Program Files\LLVM\bin;%PATH%"

REM Check LLVM
where clang.exe >nul 2>&1
if errorlevel 1 (
    echo [build_llvm_windows] ERROR: clang.exe not found
    exit /b 1
)

echo [build_llvm_windows] LLVM found, building nyash...

REM Build nyash with LLVM
cargo build --release --features llvm

if errorlevel 1 (
    echo [build_llvm_windows] ERROR: cargo build failed
    exit /b 1
)

echo [build_llvm_windows] Build successful!

REM Build ny-echo-lite
echo [build_llvm_windows] Building ny-echo-lite...
powershell -ExecutionPolicy Bypass -File tools\build_llvm.ps1 apps\tests\ny-echo-lite\main.nyash -Out app_echo.exe

echo [build_llvm_windows] Done!
endlocal