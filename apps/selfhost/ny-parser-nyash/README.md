# Ny Parser (v0) — Minimal Nyash-made Parser

- Scope: integers, + - * /, parentheses, and a single `return` statement.
- Output: JSON IR v0 as documented in CURRENT_TASK.md (Program/Return/Int/Binary).

Usage (Unix)
- echo "return 1+2*3" | ./tools/ny_parser_run.sh

Usage (Windows PowerShell)
- Get-Content .\apps\ny-mir-samples\arithmetic.nyash | .\tools\ny_parser_run.ps1

Notes
- This is a minimal educational parser to bootstrap the self-host loop.
- Errors print a JSON envelope: {"version":0,"kind":"Error",...}.
