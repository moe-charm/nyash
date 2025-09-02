@echo off
set "LLVM_SYS_180_PREFIX=C:\LLVM-18"
set "LLVM_SYS_180_FFI_WORKAROUND=1"
set "PATH=C:\LLVM-18\bin;%PATH%"

echo Building Nyash with LLVM support...
cargo build --release --features llvm

echo Done!