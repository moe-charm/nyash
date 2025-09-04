<#
  AOT smoke (Cranelift) — DRYRUN skeleton (Windows-first)
  Usage:
    pwsh -File tools/aot_smoke_cranelift.ps1 [-Mode release|debug]
  Env:
    CLIF_SMOKE_RUN=1         # actually execute steps (default: dry-run only)
    NYASH_LINK_VERBOSE=1     # echo link commands (when run)
    NYASH_DISABLE_PLUGINS=1  # plugin-dependent smokes off
  Notes:
    - This script mirrors docs/tests/aot_smoke_cranelift.md pseudo flow.
    - PoC: emits commands; real execution requires Cranelift AOT path to be implemented.
#>

param(
  [ValidateSet('release','debug')]
  [string]$Mode = 'release'
)

$Run = [int]([Environment]::GetEnvironmentVariable('CLIF_SMOKE_RUN') ?? '0')

function Banner($msg) { Write-Host "`n[clif-aot-smoke] $msg" }
function Info($msg)   { Write-Host "[clif-aot-smoke] $msg" }

$Root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$Target = Join-Path $Root 'target'
$ObjDir = Join-Path $Target 'aot_objects'
New-Item -ItemType Directory -Force -Path $ObjDir | Out-Null
$ObjOut = Join-Path $ObjDir 'core_smoke.obj'
$NyashBin = Join-Path (Join-Path $Target $Mode) 'nyash.exe'
$ExeOut = Join-Path $Target 'app_clif.exe'

Banner "Cranelift AOT Smoke (mode=$Mode, dry-run=$([bool](-not ($Run -eq 1))))"

# 1) Build nyash with cranelift
Banner "building nyash (features=cranelift-jit)"
if ($Run -eq 1) {
  & cargo build --% --$Mode --features cranelift-jit
} else {
  Info "DRYRUN: cargo build --$Mode --features cranelift-jit"
}

# 2) Emit object via backend=cranelift (PoC path)
Banner "emitting object via --backend cranelift (PoC)"
if ($Run -eq 1) {
  if (-not (Test-Path $NyashBin)) { throw "nyash not found: $NyashBin" }
  $env:NYASH_AOT_OBJECT_OUT = $ObjOut
  & $NyashBin --backend cranelift apps/hello/main.nyash | Out-Null
  if (-not (Test-Path $ObjOut)) { throw "object not generated: $ObjOut" }
  $size = (Get-Item $ObjOut).Length
  Info "OK: object generated: $ObjOut ($size bytes)"
} else {
  Info "DRYRUN: NYASH_AOT_OBJECT_OUT=$ObjOut $NyashBin --backend cranelift apps/hello/main.nyash"
  New-Item -ItemType File -Force -Path $ObjOut | Out-Null
}

# 3) Link (Windows-first)
Banner "linking app (Windows-first)"
if ($Run -eq 1) {
  $link = Get-Command link -ErrorAction SilentlyContinue
  $lld  = Get-Command lld-link -ErrorAction SilentlyContinue
  if ($link) {
    Info "using MSVC link.exe"
    & link /OUT:$ExeOut $ObjOut nyrt.lib
  } elseif ($lld) {
    Info "using lld-link"
    & lld-link -OUT:$ExeOut $ObjOut nyrt.lib
  } else {
    throw "no Windows linker found (link.exe/lld-link)"
  }
} else {
  Info "DRYRUN: link /OUT:$ExeOut $ObjOut nyrt.lib  (or lld-link)"
}

# 4) Run and verify
Banner "run and verify output"
if ($Run -eq 1) {
  if (-not (Test-Path $ExeOut)) { throw "no output binary: $ExeOut" }
  $out = & $ExeOut 2>&1
  $out | Write-Host
  if ($out -notmatch 'Result:') { throw "unexpected output" }
  Info "OK: smoke passed"
} else {
  Info "DRYRUN: $ExeOut → expect a line including: 'Result:'"
  Info "DRYRUN complete"
}

exit 0

