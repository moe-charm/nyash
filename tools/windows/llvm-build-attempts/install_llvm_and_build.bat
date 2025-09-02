@echo off
echo Installing LLVM 18.0.x via vcpkg and building Nyash...

REM Check if vcpkg exists
if not exist "C:\vcpkg\vcpkg.exe" (
    echo ERROR: vcpkg not found at C:\vcpkg\vcpkg.exe
    echo Please install vcpkg first
    exit /b 1
)

echo Step 1: Installing LLVM via vcpkg with 24 threads...
cd C:\vcpkg
vcpkg install "llvm[core]:x64-windows"

echo Step 2: Setting environment variables...
set LLVM_SYS_180_PREFIX=C:\vcpkg\installed\x64-windows
set LLVM_SYS_NO_LIBFFI=1
set LLVM_SYS_180_STRICT_VERSIONING=0
set PATH=C:\vcpkg\installed\x64-windows\bin;%PATH%

echo Step 3: Checking LLVM installation...
if exist "C:\vcpkg\installed\x64-windows\bin\llvm-config.exe" (
    echo LLVM installed successfully!
    C:\vcpkg\installed\x64-windows\bin\llvm-config.exe --version
) else (
    echo ERROR: llvm-config.exe not found
    exit /b 1
)

echo Step 4: Building Nyash with LLVM support...
cd C:\git\nyash-project\nyash
cargo clean
cargo build --release --features llvm -j24

echo Done!
pause