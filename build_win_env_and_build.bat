@echo off
setlocal ENABLEDELAYEDEXPANSION
chcp 65001 >nul

rem =============================
rem User config (編集ポイント)
rem =============================
rem LLVM のプレフィックス（ヘッダ/Core.h と CMake/LLVMConfig.cmake がある場所）
rem 例) 自前CMake: C:\LLVM-18
rem 例) vcpkg:     C:\vcpkg\installed\x64-windows
set "LLVM_PREFIX=C:\LLVM-18"

rem AOTのみなら 0（libffi無効） / JITも使うなら 1（libffi有効）
set "USE_LIBFFI=0"

rem libffi のプレフィックス（USE_LIBFFI=1 のとき使用; vcpkg推奨）
set "LIBFFI_PREFIX=C:\vcpkg\installed\x64-windows"

rem Cargo 子プロセスへ PATH を強制注入（有効化=1）
set "FORCE_CARGO_PATH=1"

rem =============================
rem VS 開発環境（MSVC x64）を有効化
rem =============================
if exist "%ProgramFiles%\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat" (
  call "%ProgramFiles%\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat" -arch=x64
) else if exist "%ProgramFiles(x86)%\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" (
  call "%ProgramFiles(x86)%\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64
)

rem =============================
rem 前提チェック
rem =============================
if not exist "%LLVM_PREFIX%\include\llvm-c\Core.h" (
  echo [ERROR] Core.h not found: "%LLVM_PREFIX%\include\llvm-c\Core.h"
  echo         LLVM_PREFIX をヘッダがある場所に直してください。
  exit /b 2
)
if not exist "%LLVM_PREFIX%\lib\cmake\llvm\LLVMConfig.cmake" (
  echo [ERROR] LLVMConfig.cmake not found: "%LLVM_PREFIX%\lib\cmake\llvm\LLVMConfig.cmake"
  echo         LLVM_PREFIX を CMake 定義がある場所に直してください。
  exit /b 2
)
if not exist "%LLVM_PREFIX%\lib\LLVMCore.lib" (
  echo [WARN] LLVMCore.lib が見つかりません: "%LLVM_PREFIX%\lib\LLVMCore.lib"
)
if not exist "%LLVM_PREFIX%\lib\LLVMSupport.lib" (
  echo [WARN] LLVMSupport.lib が見つかりません: "%LLVM_PREFIX%\lib\LLVMSupport.lib"
)

rem =============================
rem 衝突しやすい環境変数を掃除
rem =============================
set LLVM_CONFIG_PATH=
set LLVM_SYS_180_NO_LIBFFI=
set LLVM_SYS_180_FFI_WORKAROUND=

rem =============================
rem このシェル限定の環境を設定
rem =============================
set "LLVM_SYS_180_PREFIX=%LLVM_PREFIX%"
set "LLVM_SYS_180_INCLUDE_DIR=%LLVM_PREFIX%\include"
set "LLVM_SYS_180_LIB_DIR=%LLVM_PREFIX%\lib"
set "LLVM_SYS_180_STRICT_VERSIONING=0"
set "PATH=%LLVM_PREFIX%\bin;%PATH%"

if "%USE_LIBFFI%"=="1" (
  set "LLVM_SYS_NO_LIBFFI="
  if exist "%LIBFFI_PREFIX%\lib\ffi.lib" (
    set "LIB=%LIBFFI_PREFIX%\lib;%LIB%"
    set "PATH=%LIBFFI_PREFIX%\bin;%PATH%"
  ) else (
    echo [WARN] libffi not found at "%LIBFFI_PREFIX%\lib\ffi.lib" （JIT想定なら vcpkg で libffi を追加してください）
  )
) else (
  rem AOT-only
  set "LLVM_SYS_NO_LIBFFI=1"
)

rem =============================
rem 現在の設定を表示
rem =============================
echo [ENV] LLVM_SYS_180_PREFIX=%LLVM_SYS_180_PREFIX%
echo [ENV] LLVM_SYS_180_INCLUDE_DIR=%LLVM_SYS_180_INCLUDE_DIR%
echo [ENV] LLVM_SYS_180_LIB_DIR=%LLVM_SYS_180_LIB_DIR%
echo [ENV] LLVM_SYS_NO_LIBFFI=%LLVM_SYS_NO_LIBFFI%
echo [ENV] LLVM_SYS_180_STRICT_VERSIONING=%LLVM_SYS_180_STRICT_VERSIONING%
where cl
where link
where clang

rem =============================
rem Cargo 子プロセスへ PATH を強制注入（必要なら）
rem =============================
if "%FORCE_CARGO_PATH%"=="1" (
  if not exist ".cargo" mkdir ".cargo"
  > ".cargo\config.toml" (
    echo [env]
    echo LLVM_SYS_180_PREFIX = "%LLVM_SYS_180_PREFIX:\=\\%"
    echo LLVM_SYS_180_INCLUDE_DIR = "%LLVM_SYS_180_INCLUDE_DIR:\=\\%"
    echo LLVM_SYS_180_LIB_DIR = "%LLVM_SYS_180_LIB_DIR:\=\\%"
    if "%USE_LIBFFI%"=="1" (echo LLVM_SYS_NO_LIBFFI = "") else (echo LLVM_SYS_NO_LIBFFI = "1")
    echo LLVM_SYS_180_STRICT_VERSIONING = "0"
    echo PATH = { value = "%LLVM_SYS_180_PREFIX:\=\\%\bin;{PATH}", force = true }
  )
  echo [INFO] Wrote .cargo\config.toml
)

rem =============================
rem Rust toolchain を MSVC に固定＆ビルド
rem =============================
echo [INFO] Using MSVC toolchain...
rustup default stable-x86_64-pc-windows-msvc

set "RUST_LOG=llvm_sys=trace"
echo [INFO] Cleaning...
cargo clean

echo [INFO] Building nyash (release, feature=llvm)...
cargo +stable-x86_64-pc-windows-msvc build --release --features llvm -vv -j24
if errorlevel 1 (
  echo [ERROR] cargo build failed. 上の末尾（link.exe の行/ffi.libの有無）を確認してください。
  exit /b 1
)

echo [OK] Build complete.
echo [HINT] AOT→EXE: powershell -ExecutionPolicy Bypass -File tools\build_llvm.ps1 apps\tests\ny-echo-lite\main.nyash -Out app_echo.exe

endlocal
exit /b 0

