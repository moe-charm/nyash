#!/usr/bin/env pwsh
param(
  [Parameter(Mandatory=$false)][switch]$EnsureLLVM,
  [Parameter(Mandatory=$false)][switch]$SetPermanentEnv,
  [Parameter(Mandatory=$false)][string]$Profile = "release",
  [Parameter(Mandatory=$false)][string]$Target,
  [Parameter(Mandatory=$false)][string]$App,
  [Parameter(Mandatory=$false)][string]$Out = "app.exe"
)

$ErrorActionPreference = 'Stop'
function Info($m){ Write-Host "[build-nyash-llvm] $m" -ForegroundColor Cyan }
function Warn($m){ Write-Host "[build-nyash-llvm] WARN: $m" -ForegroundColor Yellow }
function Err($m){ Write-Host "[build-nyash-llvm] ERROR: $m" -ForegroundColor Red; exit 1 }

# Move to repo root for stable paths
Set-Location (Split-Path -Parent $PSCommandPath) | Out-Null
Set-Location (Resolve-Path ..\..) | Out-Null

if ($EnsureLLVM) {
  $args = @()
  if ($SetPermanentEnv) { $args += "-SetPermanent" }
  & tools\windows\ensure-llvm18.ps1 @args
}

Info "Building nyash (features=llvm, profile=$Profile)"
$cargoArgs = @('build')
if ($Profile -eq 'release') { $cargoArgs += '--release' }
elseif ($Profile -ne 'debug') { Warn "Unknown profile '$Profile', using release"; $cargoArgs += '--release' }
$cargoArgs += @('--features','llvm')
if ($Target) { $cargoArgs += @('--target', $Target) }
& cargo @cargoArgs
if ($LASTEXITCODE -ne 0) { Err "cargo build failed (llvm). Ensure LLVM_SYS_180/181_PREFIX are set and LLVM 18 is installed." }

if ($App) {
  if (-not (Test-Path $App)) { Err "App file not found: $App" }
  Info "Linking EXE via LLVM AOT: $App → $Out"
  & tools\build_llvm.ps1 $App -Out $Out
  if ($LASTEXITCODE -ne 0) { Err "link failed: $Out not produced" }
  Info "OK: built $Out"
}

Info "Done"

