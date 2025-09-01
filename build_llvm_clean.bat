@echo off
echo Cleaning environment and building with LLVM...

REM Remove old environment variables
set LLVM_CONFIG_PATH=
set LLVM_SYS_180_NO_LIBFFI=
set LLVM_SYS_180_FFI_WORKAROUND=

REM Set new environment variables
set LLVM_SYS_180_PREFIX=C:\LLVM-18
set PATH=C:\LLVM-18\bin;%PATH%
set LLVM_SYS_NO_LIBFFI=1
set LLVM_SYS_180_STRICT_VERSIONING=0
set RUST_LOG=llvm_sys=trace

echo Building with verbose output...
cargo +stable-x86_64-pc-windows-msvc build --release --features llvm -vv