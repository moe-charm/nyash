# PowerShell build script
$env:LLVM_SYS_180_PREFIX = "C:\LLVM-18"
$env:LLVM_SYS_180_FFI_WORKAROUND = "1"
$env:LLVM_SYS_NO_LIBFFI = "1"

Write-Host "Environment variables set:"
Write-Host "LLVM_SYS_180_PREFIX = $env:LLVM_SYS_180_PREFIX"
Write-Host "LLVM_SYS_NO_LIBFFI = $env:LLVM_SYS_NO_LIBFFI"

Write-Host "`nBuilding..."
cargo build --bin nyash --release --features llvm