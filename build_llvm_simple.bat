@echo off
set LLVM_SYS_180_PREFIX=C:\LLVM-18
set LLVM_SYS_NO_LIBFFI=1
set LLVM_SYS_180_STRICT_VERSIONING=0
set PATH=C:\LLVM-18\bin;%PATH%

echo LLVM_SYS_180_PREFIX=%LLVM_SYS_180_PREFIX%
echo Building with 24 threads...

cargo build --release --features llvm -j24