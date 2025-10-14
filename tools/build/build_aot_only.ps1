# PowerShell script for building Nyash with LLVM AOT support (no libffi)

Write-Host "Setting up environment for AOT-only LLVM build..." -ForegroundColor Green

# Set environment variables
$env:LLVM_SYS_180_PREFIX = "C:\LLVM-18"
$env:LLVM_SYS_180_FFI_WORKAROUND = "1"
$env:LLVM_SYS_NO_LIBFFI = "1"  # This is the key - disable libffi
$env:PATH = "C:\LLVM-18\bin;" + $env:PATH

Write-Host "Environment variables set:" -ForegroundColor Yellow
Write-Host "  LLVM_SYS_180_PREFIX = $env:LLVM_SYS_180_PREFIX"
Write-Host "  LLVM_SYS_NO_LIBFFI = $env:LLVM_SYS_NO_LIBFFI (libffi disabled for AOT)"

# Clean build directory
Write-Host "`nCleaning previous build..." -ForegroundColor Yellow
cargo clean

# Build with LLVM feature
Write-Host "`nBuilding Nyash with LLVM AOT support..." -ForegroundColor Green
cargo build --bin nyash --release --features llvm

# Check output
Write-Host "`nChecking build output..." -ForegroundColor Yellow
if (Test-Path "target\release\nyash.exe") {
    Write-Host "SUCCESS: nyash.exe created!" -ForegroundColor Green
    Get-Item "target\release\nyash.exe" | Format-List Name, Length, LastWriteTime
} else {
    Write-Host "ERROR: nyash.exe not found" -ForegroundColor Red
    Write-Host "Listing exe files in target\release:"
    Get-ChildItem "target\release\*.exe" | Format-Table Name, Length
}