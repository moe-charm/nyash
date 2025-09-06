Param()
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$bin = Join-Path $root 'target\release\nyash.exe'
$nyParser = Join-Path $root 'tools\ny_parser_run.ps1'

if (-not (Test-Path $bin)) {
  Write-Host 'Building nyash (release)...'
  cargo build --release --features cranelift-jit | Out-Null
}

Write-Host '[Roundtrip] Case A: Ny → JSON(v0) → MIR-Interp (pipe)'
$pipeOut = "return (1+2)*3`n" | & $nyParser | & $bin --ny-parser-pipe
if ($pipeOut -match '^Result:\s*9\b') { Write-Host 'PASS: Case A (pipe)' } else { Write-Host 'FAIL: Case A (pipe)'; Write-Output $pipeOut; exit 1 }

Write-Host '[Roundtrip] Case B: JSON(v0) file → MIR-Interp'
$tmp = New-TemporaryFile
@'{"version":0,"kind":"Program","body":[{"type":"Return","expr":{"type":"Binary","op":"+","lhs":{"type":"Int","value":1},"rhs":{"type":"Binary","op":"*","lhs":{"type":"Int","value":2},"rhs":{"type":"Int","value":3}}}}]}'@ | Set-Content -Path $tmp -NoNewline
$fileOut = & $bin --json-file $tmp
if ($fileOut -match '^Result:\s*7\b') { Write-Host 'PASS: Case B (json-file)' } else { Write-Host 'FAIL: Case B (json-file)'; Write-Output $fileOut; exit 1 }

Write-Host 'All PASS'
