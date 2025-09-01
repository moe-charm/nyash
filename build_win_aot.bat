@echo off
setlocal ENABLEDELAYEDEXPANSION
chcp 65001 >nul

echo [Nyash AOT Build - libffi disabled]
echo ===================================

REM Set LLVM environment variables
set "LLVM_SYS_180_PREFIX=C:\LLVM-18"
set "LLVM_SYS_NO_LIBFFI=1"
set "LLVM_SYS_180_FFI_WORKAROUND=1"

echo LLVM_SYS_180_PREFIX=%LLVM_SYS_180_PREFIX%
echo LLVM_SYS_NO_LIBFFI=%LLVM_SYS_NO_LIBFFI%

REM Verify LLVM installation
if not exist "%LLVM_SYS_180_PREFIX%\include\llvm-c\Core.h" (
    echo ERROR: Core.h not found at %LLVM_SYS_180_PREFIX%\include\llvm-c\Core.h
    exit /b 1
)

if not exist "%LLVM_SYS_180_PREFIX%\lib\cmake\llvm\LLVMConfig.cmake" (
    echo ERROR: LLVMConfig.cmake not found at %LLVM_SYS_180_PREFIX%\lib\cmake\llvm\LLVMConfig.cmake
    exit /b 1
)

echo LLVM installation verified successfully!

REM Add LLVM to PATH
set "PATH=%LLVM_SYS_180_PREFIX%\bin;%PATH%"

REM Clean and build
echo.
echo Cleaning previous build...
cargo clean

echo.
echo Building Nyash with LLVM AOT support (no libffi)...
cargo build --bin nyash --release --features llvm

REM Check result
echo.
if exist "target\release\nyash.exe" (
    echo SUCCESS: nyash.exe built successfully!
    echo.
    dir target\release\nyash.exe
) else (
    echo ERROR: Build failed - nyash.exe not found
    exit /b 1
)

endlocal