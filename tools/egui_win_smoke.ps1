# Egui (Windows) JIT smoke: build nyash + egui plugin and run demo
# Usage (PowerShell):
#   pwsh -File tools/egui_win_smoke.ps1  # or .\tools\egui_win_smoke.ps1

param(
  [switch]$DebugBuild
)

$ErrorActionPreference = 'Stop'

function Info($msg) { Write-Host "[egui-smoke] $msg" -ForegroundColor Cyan }
function Warn($msg) { Write-Host "[egui-smoke] $msg" -ForegroundColor Yellow }
function Die($msg)  { Write-Host "[egui-smoke] ERROR: $msg" -ForegroundColor Red; exit 1 }

# 1) Build nyash with Cranelift JIT
Info "Building nyash (cranelift-jit)..."
$features = @('cranelift-jit')
$mode = if ($DebugBuild) { 'debug' } else { 'release' }
$cargoArgs = @('build','--features', ($features -join ','))
if (-not $DebugBuild) { $cargoArgs += '--release' }

& cargo @cargoArgs | Out-Host
if ($LASTEXITCODE -ne 0) { Die "cargo build nyash failed" }

$nyashExe = Join-Path -Path (Resolve-Path .).Path -ChildPath ("target/{0}/nyash{1}" -f $mode, (if ($IsWindows) {'.exe'} else {''}))
if (-not (Test-Path $nyashExe)) {
  Die "nyash binary not found: $nyashExe"
}

# 2) Build the egui plugin DLL with the real window feature
Info "Building nyash-egui-plugin (with-egui)..."
$pluginArgs = @('build','-p','nyash-egui-plugin','--features','with-egui')
if (-not $DebugBuild) { $pluginArgs += '--release' }
& cargo @pluginArgs | Out-Host
if ($LASTEXITCODE -ne 0) { Die "cargo build nyash-egui-plugin failed" }

$pluginDir = Join-Path -Path (Resolve-Path .).Path -ChildPath ("plugins/nyash-egui-plugin/target/{0}" -f $mode)
if (-not (Test-Path $pluginDir)) { Die "plugin target dir not found: $pluginDir" }

# 3) Environment toggles for JIT host-bridge path
$env:NYASH_CLI_VERBOSE = '1'
$env:NYASH_MIR_CORE13 = '1'
$env:NYASH_OPT_DIAG_FORBID_LEGACY = '1'
$env:NYASH_JIT_EXEC = '1'
$env:NYASH_JIT_HOSTCALL = '1'
$env:NYASH_JIT_HOST_BRIDGE = '1'

# 4) Ensure plugin search paths include typical locations (nyash.toml already covers these)
#    Allow overriding via NYASH_PLUGIN_PATHS if the user prefers a custom path list.
if (-not $env:NYASH_PLUGIN_PATHS) {
  $env:NYASH_PLUGIN_PATHS = @(
    (Join-Path (Resolve-Path .).Path 'target\release'),
    (Join-Path (Resolve-Path .).Path 'target\debug'),
    (Join-Path (Resolve-Path .).Path 'plugins\nyash-egui-plugin\target\release'),
    (Join-Path (Resolve-Path .).Path 'plugins\nyash-egui-plugin\target\debug')
  ) -join ';'
}

# 5) Run the minimal Egui demo via Nyash script using JIT host-bridge
$appScript = Join-Path -Path (Resolve-Path .).Path -ChildPath 'apps\egui-hello\main.nyash'
if (-not (Test-Path $appScript)) { Die "demo script not found: $appScript" }

Info "Launching Egui demo (JIT host-bridge)..."
Write-Host "Command:" -ForegroundColor DarkGray
Write-Host "  `"$nyashExe`" --backend vm --jit-exec --jit-hostcall `"$appScript`"" -ForegroundColor DarkGray

& $nyashExe --backend vm --jit-exec --jit-hostcall $appScript
$code = $LASTEXITCODE
if ($code -ne 0) {
  Warn "nyash exited with code $code"
} else {
  Info "Run finished (exit code 0). If on Windows, a window should have appeared."
}

Info "Done."

