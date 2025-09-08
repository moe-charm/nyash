<#
  One‑shot manual build for app_egui.exe (Windows, PowerShell)

  This script replicates the exact manual steps you listed:
   1) Build Egui plugin (with-egui)
   2) Build Nyash (with Cranelift AOT toolchain)
   3) Emit AOT object (main.o) from the Egui Nyash script
      NOTE: An Egui window will open — close it to continue
   4) Build the NyRT static runtime library
   5) Link main.o + NyRT into app_egui.exe using clang (or cc)

  Usage:
    pwsh -File tools/windows/build_app_egui_manual.ps1 \
      -Input apps/egui-hello-plugin/main.nyash -Out app_egui.exe

  Options:
    -Input  : Path to Nyash Egui script (default: apps/egui-hello-plugin/main.nyash)
    -Out    : Output exe path/name (default: app_egui.exe)
    -Verbose: Prints extra logs
#>

param(
  [Alias('Input')][string]$InputPath = "apps/egui-hello-plugin/main.nyash",
  [Alias('Out')][string]$OutputExe  = "app_egui.exe",
  [switch]$Verbose
)

$ErrorActionPreference = 'Stop'

function Info($msg) { Write-Host "[manual] $msg" -ForegroundColor Cyan }
function Warn($msg) { Write-Host "[manual] $msg" -ForegroundColor Yellow }
function Fail($msg) { Write-Host "[manual] ERROR: $msg" -ForegroundColor Red; exit 1 }

if ($Verbose) { $env:NYASH_CLI_VERBOSE = '1' }

# Normalize/resolve paths
try { $InputPath = (Resolve-Path $InputPath).Path } catch { Fail "Input script not found: $InputPath" }
if (-not [System.IO.Path]::IsPathRooted($OutputExe)) { $OutputExe = (Join-Path (Get-Location) $OutputExe) }

# 1) Egui plugin (with-egui)
Info "Building Egui plugin (with-egui)..."
Push-Location plugins/nyash-egui-plugin
try {
  cargo build --release --features with-egui | Out-Host
} finally { Pop-Location }

# 2) Nyash core (Cranelift tooling enabled)
Info "Building Nyash (cranelift-jit feature for AOT tools)..."
cargo build --release --features cranelift-jit | Out-Host

# 3) Emit main.o via Nyash (AOT object)
$env:NYASH_AOT_OBJECT_OUT = if ([string]::IsNullOrWhiteSpace($env:NYASH_AOT_OBJECT_OUT)) { "target/aot_objects" } else { $env:NYASH_AOT_OBJECT_OUT }
if (-not (Test-Path $env:NYASH_AOT_OBJECT_OUT)) { [void][System.IO.Directory]::CreateDirectory($env:NYASH_AOT_OBJECT_OUT) }
Remove-Item Env:NYASH_OPT_DIAG_FORBID_LEGACY -ErrorAction SilentlyContinue
$env:NYASH_SKIP_TOML_ENV = '1'

# Minimal strictness to keep emission deterministic
$env:NYASH_USE_PLUGIN_BUILTINS = '1'
$env:NYASH_JIT_EXEC = '1'
$env:NYASH_JIT_ONLY = '1'
$env:NYASH_JIT_STRICT = '1'
$env:NYASH_JIT_ARGS_HANDLE_ONLY = '1'
$env:NYASH_JIT_THRESHOLD = '1'

Info "Emitting main.o (an Egui window will appear — close it to continue)..."
& .\target\release\nyash.exe --backend vm $InputPath | Out-Null

$obj = Join-Path $env:NYASH_AOT_OBJECT_OUT 'main.o'
if (-not (Test-Path $obj)) { Fail "object not generated: $obj" }

# 4) Build NyRT static runtime
Info "Building NyRT static runtime..."
Push-Location crates/nyrt
try {
  cargo build --release | Out-Host
} finally { Pop-Location }

# 5) Link
Info "Linking $OutputExe ..."
# Search for NyRT static library in common locations (workspace or crate-local)
$candidateDirs = @("target/release", "crates/nyrt/target/release")
$libPath = $null
foreach ($d in $candidateDirs) {
  $p1 = Join-Path $d "nyrt.lib"
  $p2 = Join-Path $d "libnyrt.a"
  if (Test-Path $p1) { $libPath = $p1; break }
  if (Test-Path $p2) { $libPath = $p2; break }
}
if ($null -eq $libPath) { Fail "NyRT static library not found under: $($candidateDirs -join ', ')" }

# Prefer specific LLVM clang if present
$clangCandidates = @(
  "$Env:LLVM_SYS_180_PREFIX\bin\clang.exe",
  "C:\\LLVM-18\\bin\\clang.exe",
  (Get-Command clang -ErrorAction SilentlyContinue | ForEach-Object { $_.Source })
) | Where-Object { $_ -and (Test-Path $_) }

if ($clangCandidates.Count -gt 0) {
  $clang = $clangCandidates[0]
  Info "Using clang: $clang"
  & $clang $obj $libPath -lUser32 -lGdi32 -lShell32 -lOle32 -lAdvapi32 -lWs2_32 -lNtdll -o $OutputExe | Out-Host
} else {
  # Fallback: use bash/cc with Linux-like flags, if available (MSYS2/WSL)
  $bash = Get-Command bash -ErrorAction SilentlyContinue
  if ($bash) {
    & bash -lc "cc '$obj' -L '$libDir' -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive -lpthread -ldl -lm -o '$OutputExe'" | Out-Host
  } else {
    Fail "Neither clang nor bash/cc found. Install LLVM clang or MSYS2/WSL toolchain."
  }
}

if (Test-Path $OutputExe) {
  Info "Success. Output: $OutputExe"
  Write-Host "Run: $OutputExe"
} else {
  Fail "Output exe not found: $OutputExe"
}
