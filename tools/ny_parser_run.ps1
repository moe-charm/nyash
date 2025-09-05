Param()
$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$root = Join-Path $here '..' | Resolve-Path

& (Join-Path $root 'target\release\nyash.exe') (Join-Path $root 'apps\ny-parser-nyash\main.nyash')
