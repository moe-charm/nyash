Param(
  [Parameter(Mandatory=$true, Position=0)][string]$Input,
  [Parameter(Mandatory=$false)][string]$Out = "app.exe"
)
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Info($m) { Write-Host "[AOT] $m" }
function Fail($m) { Write-Host "error: $m" -ForegroundColor Red; exit 1 }

if (-not (Test-Path $Input)) { Fail "input file not found: $Input" }

Info "Building nyash (Cranelift)..."
& cargo build --release --features cranelift-jit | Out-Null

Info "Emitting object (.o) via JIT (Strict/No-fallback)..."
$env:NYASH_AOT_OBJECT_OUT = "target/aot_objects"
$env:NYASH_USE_PLUGIN_BUILTINS = "1"
$env:NYASH_JIT_EXEC = "1"
$env:NYASH_JIT_ONLY = "1"
$env:NYASH_JIT_STRICT = "1"
$env:NYASH_JIT_ARGS_HANDLE_ONLY = "1"
$env:NYASH_JIT_THRESHOLD = "1"
New-Item -ItemType Directory -Force -Path $env:NYASH_AOT_OBJECT_OUT | Out-Null
& .\target\release\nyash --backend vm $Input | Out-Null

$OBJ = "target/aot_objects/main.o"
if (-not (Test-Path $OBJ)) {
  Fail "object not generated: $OBJ`n  hint: ensure main() is lowerable under current Strict JIT coverage"
}

Info "Building libnyrt (static runtime)..."
Push-Location crates\nyrt
& cargo build --release | Out-Null
Pop-Location

Info "Linking $Out ..."

# Try native clang first (LLVM for Windows). On Windows, we avoid -lpthread/-ldl/-lm.
$clang = Get-Command clang -ErrorAction SilentlyContinue
if ($clang) {
  $libDir = "crates/nyrt/target/release"
  $libName = ""
  if (Test-Path (Join-Path $libDir "nyrt.lib")) { $libName = "nyrt.lib" }
  elseif (Test-Path (Join-Path $libDir "libnyrt.a")) { $libName = "libnyrt.a" }
  if ($libName -ne "") {
    & clang $OBJ -L $libDir -Wl,--whole-archive -l:$libName -Wl,--no-whole-archive -o $Out | Out-Null
  }
}

if (-not (Test-Path $Out)) {
  $bash = Get-Command bash -ErrorAction SilentlyContinue
  if ($bash) {
    # Prefer WSL/MSYS2 bash to reuse Linux-like flags if available
    & bash -lc "cc target/aot_objects/main.o -L crates/nyrt/target/release -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive -lpthread -ldl -lm -o $Out" | Out-Null
  }
}

if (-not (Test-Path $Out)) {
  Write-Warning "Link step could not produce $Out."
  Write-Host "hint: Install LLVM clang (preferred) or MSYS2 toolchain, or link manually:"
  Write-Host "      clang target/aot_objects/main.o -L crates/nyrt/target/release -Wl,--whole-archive -l:libnyrt.a -Wl,--no-whole-archive -o app.exe"
  Fail "automatic link not available on this environment"
}

Info "Done: $Out"
Write-Host "   (runtime requires nyash.toml and plugin .so/.dll per config)"
