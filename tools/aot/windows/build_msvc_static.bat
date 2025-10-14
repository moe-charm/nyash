@echo off
setlocal enableextensions enabledelayedexpansion

REM Quicklink: Build and link a tiny ny_main against hako_kernel.lib (MSVC)
REM Prereqs:
REM   - MSVC toolchain (Developer Command Prompt or vcvarsall)
REM   - LLVM clang.exe in PATH (recommended)
REM   - Rust-built static runtime: target\x86_64-pc-windows-msvc\release\hako_kernel.lib

set LLVM_CLANG=%LLVM_CLANG%
if "%LLVM_CLANG%"=="" set LLVM_CLANG=clang.exe

set KERNEL_LIB=%1
if "%KERNEL_LIB%"=="" set KERNEL_LIB=target\x86_64-pc-windows-msvc\release\hako_kernel.lib

set OUT_EXE=%2
if "%OUT_EXE%"=="" set OUT_EXE=build\test_msvc.exe

if not exist build mkdir build >nul 2>&1

echo [1/2] Compiling ny_main (clang)...
"%LLVM_CLANG%" --target=x86_64-pc-windows-msvc -c tools\aot\windows\ny_main_win.c -o build\ny_main_win.obj || goto :eof

echo [2/2] Linking %OUT_EXE% ...
"%LLVM_CLANG%" build\ny_main_win.obj "%KERNEL_LIB%" ^
  ws2_32.lib advapi32.lib userenv.lib ole32.lib bcrypt.lib user32.lib kernel32.lib ^
  -Wl,/FORCE:MULTIPLE ^
  -o "%OUT_EXE%" || goto :eof

echo OK: %OUT_EXE%
exit /b 0

