#!/usr/bin/env bash
# Windows profile environment configuration

# Extended timeout for Windows builds (cross-platform overhead)
export SMOKES_DEFAULT_TIMEOUT=${SMOKES_DEFAULT_TIMEOUT:-180}

# Windows-specific paths (overridable)
export WINDOWS_LLVM_ROOT="${WINDOWS_LLVM_ROOT:-C:\Program Files\LLVM}"
export WINDOWS_BUILD_SCRIPT="${WINDOWS_BUILD_SCRIPT:-tools\build_llvm.ps1}"

# WSL interop check
if ! command -v powershell.exe >/dev/null 2>&1; then
    export WINDOWS_AVAILABLE=0
else
    export WINDOWS_AVAILABLE=1
fi
