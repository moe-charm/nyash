param(
  [Alias('Exe')][string]$ExePath = "app_egui.exe",
  [switch]$Verbose
)

$ErrorActionPreference = 'Stop'
function Info($m) { Write-Host "[run-egui] $m" -ForegroundColor Cyan }
function Die($m)  { Write-Host "[run-egui] ERROR: $m" -ForegroundColor Red; exit 1 }

if ($Verbose) { $env:NYASH_CLI_VERBOSE = '1' }
# Extra plugin/loader diagnostics
$env:NYASH_DEBUG_PLUGIN = '1'

# Ensure plugin paths (nyash.toml covers these, but make it explicit to avoid CWD issues)
$root = (Resolve-Path .).Path
$paths = @(
  (Join-Path $root 'target\release'),
  (Join-Path $root 'plugins\nyash-egui-plugin\target\release')
)
if (-not $env:NYASH_PLUGIN_PATHS) { $env:NYASH_PLUGIN_PATHS = ($paths -join ';') }

# Resolve exe absolute path
try { $exeAbs = (Resolve-Path -LiteralPath $ExePath).Path } catch { Die "exe not found: $ExePath" }
if (-not (Test-Path -LiteralPath $exeAbs)) { Die "exe not found: $exeAbs" }

# Prepare log files
$logDir = Join-Path $root 'logs'; if (-not (Test-Path $logDir)) { [void][IO.Directory]::CreateDirectory($logDir) }
$outLog = Join-Path $logDir 'app_egui_stdout.log'
$errLog = Join-Path $logDir 'app_egui_stderr.log'
if (Test-Path $outLog) { Remove-Item $outLog -Force }
if (Test-Path $errLog) { Remove-Item $errLog -Force }

Info "Launching $exeAbs ... (logs: $outLog, $errLog)"
try {
  $p = Start-Process -FilePath $exeAbs -PassThru -Wait -RedirectStandardOutput $outLog -RedirectStandardError $errLog
  Info ("ExitCode = {0}" -f $p.ExitCode)
} catch {
  Write-Host $_ -ForegroundColor Red
  Die "failed to start: $exeAbs"
}

Write-Host "--- tail stdout ---" -ForegroundColor DarkGray
if (Test-Path $outLog) { Get-Content $outLog -Tail 50 | Write-Host }
Write-Host "--- tail stderr ---" -ForegroundColor DarkGray
if (Test-Path $errLog) { Get-Content $errLog -Tail 50 | Write-Host }

Read-Host "[run-egui] Press Enter to close"

