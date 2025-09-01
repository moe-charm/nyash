#!/usr/bin/env pwsh
param(
  [Parameter(Mandatory=$true, Position=0)][string]$NyashFile,
  [Parameter(Mandatory=$false)][string]$Out = "app.exe"
)

# Build + link AOT on Windows (MSVC or MinGW)
# Requirements:
# - LLVM 18 installed (clang/lld in PATH). Optionally set LLVM_SYS_180_PREFIX
# - Rust toolchain for Windows (default host)
# - This repo builds with `--features llvm`

$ErrorActionPreference = 'Stop'

function Info($msg) { Write-Host "[build-llvm.ps1] $msg" -ForegroundColor Cyan }
function Err($msg) { Write-Host "[build-llvm.ps1] ERROR: $msg" -ForegroundColor Red; exit 1 }

# Ensure object dir exists
$objDir = Join-Path $PSScriptRoot "..\target\aot_objects"
New-Item -ItemType Directory -Path $objDir -Force | Out-Null
$objPath = Join-Path $objDir ("{0}.o" -f ([IO.Path]::GetFileNameWithoutExtension($Out)))

# Build nyash with LLVM backend
Info "Building nyash (release, feature=llvm)"
if ($env:LLVM_SYS_181_PREFIX) { Info "LLVM_SYS_181_PREFIX=$($env:LLVM_SYS_181_PREFIX)" }
elseif ($env:LLVM_SYS_180_PREFIX) { Info "LLVM_SYS_180_PREFIX=$($env:LLVM_SYS_180_PREFIX)" }
cargo build --release --features llvm | Out-Null

# Emit object from the Nyash program
Remove-Item -ErrorAction SilentlyContinue $objPath
Info "Emitting object: $objPath from $NyashFile"
$env:NYASH_LLVM_OBJ_OUT = (Resolve-Path $objPath)
if (-not $env:LLVM_SYS_181_PREFIX -and $env:LLVM_SYS_180_PREFIX) { $env:LLVM_SYS_181_PREFIX = $env:LLVM_SYS_180_PREFIX }
& .\target\release\nyash.exe --backend llvm $NyashFile | Out-Null
if (!(Test-Path $objPath)) { Err "Object not generated: $objPath" }
if ((Get-Item $objPath).Length -le 0) { Err "Object is empty: $objPath" }

# Build NyRT static library for current host
Info "Building NyRT (static lib)"
cargo build -p nyrt --release | Out-Null

# Try MSVC first (.lib), then MinGW (.a)
$nyrtLibMSVC = Join-Path $PSScriptRoot "..\target\release\nyrt.lib"
$nyrtLibGNU  = Join-Path $PSScriptRoot "..\target\release\libnyrt.a"

if (Test-Path $nyrtLibMSVC) {
  Info "Linking (MSVC): $Out"
  # Use clang/lld to link COFF obj + .lib
  & clang -fuse-ld=lld -o $Out $objPath $nyrtLibMSVC 2>$null
}
elseif (Test-Path $nyrtLibGNU) {
  Info "Linking (MinGW): $Out"
  & clang -o $Out $objPath $nyrtLibGNU -static 2>$null
}
else {
  Err "NyRT static library not found (expected nyrt.lib or libnyrt.a)"
}

if (!(Test-Path $Out)) { Err "Link failed: $Out not found" }
Info ("OK: built {0} ({1} bytes)" -f $Out, (Get-Item $Out).Length)
