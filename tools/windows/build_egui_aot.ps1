# NOTE: Save this file with ANSI encoding (no BOM). Use only ASCII characters in this file.

param(
  [Alias('Input')][string]$InputPath = "apps/egui-hello-plugin/main.nyash",
  [Alias('Out')][string]$OutputPath = "app_egui",
  [switch]$Verbose
)

function Info($msg) { Write-Host "[build] $msg" }
function Fail($msg) { Write-Host "[error] $msg"; exit 1 }

$ErrorActionPreference = "Stop"

if ($Verbose) { $env:NYASH_CLI_VERBOSE = "1" }

# Normalize paths
if ([string]::IsNullOrWhiteSpace($InputPath)) { Fail "Input is empty. Example: -Input .\apps\egui-hello-plugin\main.nyash" }
if ([string]::IsNullOrWhiteSpace($OutputPath)) { $OutputPath = "app_egui" }
try { $InputPath = (Resolve-Path $InputPath).Path } catch { Fail "Input script not found: $InputPath" }
if (-not [System.IO.Path]::IsPathRooted($OutputPath)) { $OutputPath = (Join-Path (Get-Location) $OutputPath) }
if (-not $OutputPath.ToLower().EndsWith('.exe')) { $OutputExe = "$OutputPath.exe" } else { $OutputExe = $OutputPath }
Info "Input=$InputPath"
Info "Out=$OutputExe"

# 1) Build Egui plugin (with-egui)
Info "Building Egui plugin (with-egui)..."
Push-Location plugins/nyash-egui-plugin
try {
  cargo build --release --features with-egui | Out-Host
} catch {
  Pop-Location
  Fail "Plugin build failed"
}
Pop-Location

# 2) Build nyash with Cranelift (AOT tools)
Info "Building nyash (cranelift-jit feature for AOT tools)..."
try {
  cargo build --release --features cranelift-jit | Out-Host
} catch {
  Fail "nyash build failed"
}

# 3) AOT: emit object (.o) via JIT-direct (not VM)
Info "Emitting object (.o) via JIT-direct..."
$host.ui.WriteLine("[build] Heads-up: Running Nyash (jit-direct) to emit main.o will open the Egui window. Close it to continue.")
$env:NYASH_AOT_OBJECT_OUT = if ([string]::IsNullOrWhiteSpace($env:NYASH_AOT_OBJECT_OUT)) { "target/aot_objects" } else { $env:NYASH_AOT_OBJECT_OUT }
if (-not (Test-Path $env:NYASH_AOT_OBJECT_OUT)) { [void][System.IO.Directory]::CreateDirectory($env:NYASH_AOT_OBJECT_OUT) }
$env:NYASH_PLUGIN_ONLY = "1"
$env:NYASH_JIT_EXEC = "1"
& .\target\release\nyash --jit-direct $InputPath | Out-Host

$OBJ = Join-Path $env:NYASH_AOT_OBJECT_OUT "main.o"
if (-not (Test-Path $OBJ)) {
  Fail "object not generated: $OBJ`n  hint: ensure main() is lowerable under current Strict JIT coverage"
}

Info "Building libnyrt (static runtime)..."
Push-Location crates\nyrt
& cargo build --release | Out-Null
Pop-Location

Info "Linking $OutputExe ..."

$clang = Get-Command clang -ErrorAction SilentlyContinue
if ($clang) {
  # Search for nyrt static lib in both workspace root and crate-local targets
  $candidateDirs = @("target/release", "crates/nyrt/target/release")
  $libPath = $null
  foreach ($d in $candidateDirs) {
    $p1 = Join-Path $d "nyrt.lib"
    $p2 = Join-Path $d "libnyrt.a"
    if (Test-Path $p1) { $libPath = $p1; break }
    if (Test-Path $p2) { $libPath = $p2; break }
  }
  if ($null -ne $libPath) {
    # On Windows, avoid -lpthread/-ldl/-lm; add common Win32 libs
    $args = @(
      $OBJ, $libPath,
      "-lUser32", "-lGdi32", "-lShell32", "-lOle32", "-lAdvapi32", "-lWs2_32", "-lNtdll",
      "-o", $OutputExe
    )
    & clang @args | Out-Null
  } else {
    Write-Host "[build] nyrt library not found in: $($candidateDirs -join ', ')" -ForegroundColor Yellow
  }
}

if (Test-Path "$OutputExe") {
  Info "Success. Output: $OutputExe"
  Write-Host "Run: $OutputExe"
} else {
  Fail "Output exe not found: $OutputExe"
}
