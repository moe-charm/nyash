@echo off
set "LLVM_SYS_180_PREFIX=C:\LLVM-18"
set "LLVM_SYS_180_FFI_WORKAROUND=1"

echo Checking build status...
dir target\release\*.exe 2>nul
if errorlevel 1 (
    echo No exe files in target\release
    dir target\x86_64-pc-windows-msvc\release\*.exe 2>nul
    if errorlevel 1 (
        echo No exe files in Windows target either
    )
)