@echo off
echo Setting system-wide environment variables...

REM Set system environment variables (requires admin)
setx LLVM_SYS_180_PREFIX "C:\LLVM-18" /M
setx LLVM_SYS_180_FFI_WORKAROUND "1" /M  
setx LLVM_SYS_NO_LIBFFI "1" /M

echo.
echo System environment variables set!
echo Please restart your command prompt for changes to take effect.
echo.
echo For user-level variables (no admin required):
setx LLVM_SYS_180_PREFIX "C:\LLVM-18"
setx LLVM_SYS_180_FFI_WORKAROUND "1"
setx LLVM_SYS_NO_LIBFFI "1"

echo.
echo User environment variables also set!
pause