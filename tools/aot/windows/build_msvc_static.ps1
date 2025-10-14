# Quicklink: Build and link a tiny ny_main against hako_kernel.lib (MSVC)
# Prereqs:
#   - Developer Command Prompt for VS or vcvarsall configured
#   - LLVM clang.exe available in PATH (recommended for asm alias support)
#   - Rust-built static runtime: target\x86_64-pc-windows-msvc\release\hako_kernel.lib

param(
  [string]$KernelLib = "target\x86_64-pc-windows-msvc\release\hako_kernel.lib",
  [string]$OutExe   = "build\test_msvc.exe"
)

$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force -Path build | Out-Null

Write-Host "[1/2] Compiling ny_main (clang) ..."
clang.exe --target=x86_64-pc-windows-msvc -c tools\aot\windows\ny_main_win.c -o build\ny_main_win.obj

Write-Host "[2/2] Linking $OutExe ..."
clang.exe build\ny_main_win.obj $KernelLib `
  ws2_32.lib advapi32.lib userenv.lib ole32.lib bcrypt.lib user32.lib kernel32.lib `
  -Wl,/FORCE:MULTIPLE `
  -o $OutExe

Write-Host "OK: $OutExe"
