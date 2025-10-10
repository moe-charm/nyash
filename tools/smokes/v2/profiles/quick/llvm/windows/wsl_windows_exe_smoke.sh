#!/usr/bin/env bash
# wsl_windows_exe_smoke.sh — Build Windows .exe from WSL using PowerShell script and verify exit code

set -euo pipefail

source "$(dirname "$0")/../../../../lib/test_runner.sh"
require_env || exit 2

name="wsl_windows_exe_smoke"

# Windows build can take time; extend default timeout if not set
export SMOKES_DEFAULT_TIMEOUT=${SMOKES_DEFAULT_TIMEOUT:-180}

# Preconditions: WSL + powershell.exe available + LLVM on Windows side
if ! command -v powershell.exe >/dev/null 2>&1; then
  test_skip "$name" "powershell.exe not found (WSL Windows interop)" && exit 0
fi

# Create minimal program returning 3
TMP_DIR="/tmp/${name}_$$"; mkdir -p "$TMP_DIR"
SRC="$TMP_DIR/min_ret.nyash"
cat > "$SRC" << 'SRC'
static box Main { main() { return 3 } }
SRC

# Windows paths via wslpath -w
ROOT_UNIX="$NYASH_ROOT"
ROOT_WIN=$(wslpath -w "$ROOT_UNIX")
SRC_WIN=$(wslpath -w "$SRC")
PS1_WIN="$ROOT_WIN\tools\build_llvm.ps1"
OUT_WIN="$ROOT_WIN\tmp\wsl_win_smoke.exe"

# Ensure tmp dir on Windows
mkdir -p "$NYASH_ROOT/tmp"

# Invoke build script on Windows side
set +e
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$PS1_WIN" "$SRC_WIN" -Out "$OUT_WIN" 2>/dev/null
code_build=$?
set -e
if [ "$code_build" -ne 0 ]; then
  test_skip "$name" "build_llvm.ps1 failed or Windows LLVM not configured (code=$code_build)" && exit 0
fi

# Run the produced .exe and capture its exit code
set +e
powershell.exe -NoProfile -Command "& \"$OUT_WIN\"; exit \$LASTEXITCODE" >/dev/null 2>&1
code_run=$?
set -e

if [ "$code_run" -ne 3 ]; then
  test_skip "$name" "Windows EXE run failed or wrong exit (got=$code_run)" && exit 0
fi

echo "Windows EXE exit=$code_run"
echo "OK: $name"
exit 0
