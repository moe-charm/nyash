@echo off
echo Installing vcpkg and LLVM...

REM Clone vcpkg if not exists
if not exist "C:\vcpkg" (
    echo Cloning vcpkg...
    cd C:\
    git clone https://github.com/Microsoft/vcpkg.git
)

REM Bootstrap vcpkg
cd C:\vcpkg
echo Bootstrapping vcpkg...
call bootstrap-vcpkg.bat

REM Integrate vcpkg with Visual Studio
echo Integrating vcpkg...
vcpkg integrate install

REM Install LLVM and libffi
echo Installing LLVM and libffi via vcpkg (this may take a while)...
vcpkg install llvm[clang]:x64-windows
vcpkg install libffi:x64-windows

echo.
echo Installation complete!
echo LLVM should be installed at: C:\vcpkg\installed\x64-windows
echo.
pause