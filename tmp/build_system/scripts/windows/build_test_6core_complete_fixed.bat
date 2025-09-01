@echo off
chcp 65001 > nul
echo === Test_6Core_Complete Windows Build ===

call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"

set CMAKE_PREFIX_PATH=C:\Qt\6.9.1\msvc2022_64
set Qt6_DIR=C:\Qt\6.9.1\msvc2022_64\lib\cmake\Qt6

if not exist build-test-6core-complete mkdir build-test-6core-complete
cd build-test-6core-complete

copy "..\CMakeLists_Test_6Core.txt" ".\CMakeLists.txt"

cmake . -G "NMake Makefiles" -DCMAKE_BUILD_TYPE=Release -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% -DQt6_DIR=%Qt6_DIR%

if %errorlevel% neq 0 (
    echo CMake configuration failed
    exit /b 1
)

nmake Test_6Core_Complete

if %errorlevel% neq 0 (
    echo Build failed
    exit /b 1
)

if exist "Test_6Core_Complete.exe" (
    if not exist "..\windows_exe" mkdir "..\windows_exe"
    copy /Y "Test_6Core_Complete.exe" "..\windows_exe\"
    echo SUCCESS: Test_6Core_Complete.exe created and copied
) else (
    echo ERROR: Test_6Core_Complete.exe not found
    exit /b 1
)