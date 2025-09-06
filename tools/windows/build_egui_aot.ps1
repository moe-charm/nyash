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

# 3) AOT: emit native exe - MERGED FROM build_aot.ps1
Info "Emitting object (.o) via JIT (Strict/No-fallback)..."
$host.ui.WriteLine("[build] Heads-up: Running Nyash to emit main.o will open the Egui window. Close the window to continue linking.")
$env:NYASH_AOT_OBJECT_OUT = if ([string]::IsNullOrWhiteSpace($env:NYASH_AOT_OBJECT_OUT)) { "target/aot_objects" } else { $env:NYASH_AOT_OBJECT_OUT }
$env:NYASH_USE_PLUGIN_BUILTINS = "1"
$env:NYASH_JIT_EXEC = "1"
$env:NYASH_JIT_ONLY = "1"
$env:NYASH_JIT_STRICT = "1"
$env:NYASH_JIT_ARGS_HANDLE_ONLY = "1"
$env:NYASH_JIT_THRESHOLD = "1"
if (-not (Test-Path $env:NYASH_AOT_OBJECT_OUT)) { [void][System.IO.Directory]::CreateDirectory($env:NYASH_AOT_OBJECT_OUT) }
& .\target\release\nyash --backend vm $InputPath | Out-Null

$OBJ = Join-Path $env:NYASH_AOT_OBJECT_OUT "main.o"
if (-not (Test-Path $OBJ)) {
  Fail "object not generated: $OBJ`n  hint: ensure main() is lowerable under current Strict JIT coverage"
}

Info "Building libnyrt (static runtime)..."
Push-Location crates\nyrt
& cargo build --release | Out-Null
Pop-Location

Info "Linking $OutputExe ..."

# Try native clang first (LLVM for Windows). On Windows, we avoid -lpthread/-ldl/-lm.
$clang = Get-Command clang -ErrorAction SilentlyContinue
if ($clang) {
  $libDir = "crates/nyrt/target/release"
  $libName = ""
  if (Test-Path (Join-Path $libDir "nyrt.lib")) { $libName = "nyrt.lib" }
  elseif (Test-Path (Join-Path $libDir "libnyrt.a")) { $libName = "libnyrt.a" }
  if ($libName -ne "") {
    $libPath = Join-Path $libDir $libName
    $args = @($OBJ, $libPath, "-o", $OutputExe)
    & clang @args | Out-Null
  }
}

if (-not (Test-Path $OutputExe)) {
  $bash = Get-Command bash -ErrorAction SilentlyContinue
  if ($bash) {
    & bash -lc "cc target/aot_objects/main.o -L crates/nyrt/target/release -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive -lpthread -ldl -lm -o $OutputExe" | Out-Null
  }
}

if (Test-Path "$OutputExe") {
  Info "Success. Output: $OutputExe"
  Write-Host "Run: $OutputExe"
} else {
  Fail "Output exe not found: $OutputExe"
}
