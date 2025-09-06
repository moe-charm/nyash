Param()
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$bin = Join-Path $root 'target\release\nyash.exe'

if (-not (Test-Path $bin)) {
  Write-Host 'Building nyash (release)...'
  cargo build --release --features cranelift-jit | Out-Null
}

Write-Host '[Smoke] Parser v0 JSON pipe → MIR-Interp'
$json = '{"version":0,"kind":"Program","body":[{"type":"Return","expr":{"type":"Binary","op":"+","lhs":{"type":"Int","value":1},"rhs":{"type":"Binary","op":"*","lhs":{"type":"Int","value":2},"rhs":{"type":"Int","value":3}}}}]}'
$pipeOut = $json | & $bin --ny-parser-pipe
if ($pipeOut -match 'Result:') { Write-Host 'PASS: pipe path' } else { Write-Host 'FAIL: pipe path'; Write-Output $pipeOut; exit 1 }

Write-Host '[Smoke] --json-file path'
$tmp = New-TemporaryFile
@'{"version":0,"kind":"Program","body":[{"type":"Return","expr":{"type":"Binary","op":"+","lhs":{"type":"Int","value":1},"rhs":{"type":"Binary","op":"*","lhs":{"type":"Int","value":2},"rhs":{"type":"Int","value":3}}}}]}'@ | Set-Content -Path $tmp -NoNewline
$fileOut = & $bin --json-file $tmp
if ($fileOut -match 'Result:') { Write-Host 'PASS: json-file path' } else { Write-Host 'FAIL: json-file path'; Write-Output $fileOut; exit 1 }
Write-Host 'All PASS'
