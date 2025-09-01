@echo off
echo Running PowerShell build script for AOT-only LLVM...
powershell -ExecutionPolicy Bypass -File build_aot_only.ps1
pause