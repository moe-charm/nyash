@echo off
setx LLVM_SYS_180_PREFIX "C:\LLVM-18"
setx LLVM_SYS_180_FFI_WORKAROUND "1"
setx LLVM_SYS_NO_LIBFFI "1"
echo Environment variables set. Please restart PowerShell and run: cargo build --release --features llvm