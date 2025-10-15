$env:LLVM_SYS_180_PREFIX = "C:\LLVM-18"
$env:LLVM_SYS_180_FFI_WORKAROUND = "1"
$env:LLVM_SYS_NO_LIBFFI = "1"
cargo build --release --features llvm