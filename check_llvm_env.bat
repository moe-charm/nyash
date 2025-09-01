@echo off
echo === LLVM Environment Check ===
echo.
echo LLVM_SYS_180_PREFIX = %LLVM_SYS_180_PREFIX%
echo.

echo Checking if LLVM files exist...
if exist "C:\LLVM-18\include\llvm-c\Core.h" (
    echo [OK] Core.h found at C:\LLVM-18\include\llvm-c\Core.h
) else (
    echo [ERROR] Core.h NOT FOUND
)

if exist "C:\LLVM-18\lib\cmake\llvm\LLVMConfig.cmake" (
    echo [OK] LLVMConfig.cmake found at C:\LLVM-18\lib\cmake\llvm\LLVMConfig.cmake
) else (
    echo [ERROR] LLVMConfig.cmake NOT FOUND
)

echo.
echo Setting environment variables...
set "LLVM_SYS_180_PREFIX=C:\LLVM-18"
set "LLVM_SYS_180_FFI_WORKAROUND=1"
set "LLVM_SYS_NO_LIBFFI=1"

echo.
echo After setting:
echo LLVM_SYS_180_PREFIX = %LLVM_SYS_180_PREFIX%
echo LLVM_SYS_180_FFI_WORKAROUND = %LLVM_SYS_180_FFI_WORKAROUND%
echo LLVM_SYS_NO_LIBFFI = %LLVM_SYS_NO_LIBFFI%

echo.
echo Testing cargo environment...
cargo --version
rustc --version

pause