@echo off
echo === Test_6Core_Complete Windows Build v2 ===

call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
if %errorlevel% neq 0 (
    echo ERROR: Visual Studio 2022 environment setup failed
    exit /b 1
)

set CMAKE_PREFIX_PATH=C:\Qt\6.9.1\msvc2022_64
set Qt6_DIR=C:\Qt\6.9.1\msvc2022_64\lib\cmake\Qt6

if not exist build-test-6core-complete mkdir build-test-6core-complete
cd build-test-6core-complete

echo Starting CMake configuration...

copy /Y "..\CMakeLists_Test_6Core.txt" "..\CMakeLists.txt"

cmake .. -G "NMake Makefiles" -DCMAKE_BUILD_TYPE=Release -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% -DQt6_DIR=%Qt6_DIR%

if %errorlevel% neq 0 (
    echo ERROR: CMake configuration failed
    cd ..
    exit /b 1
)

echo Starting build execution...

cmake --build . --config Release --target Test_6Core_Complete

if %errorlevel% neq 0 (
    echo ERROR: Build failed
    cd ..
    exit /b 1
)

echo Copying executable to windows_exe directory...
if not exist "..\windows_exe" mkdir "..\windows_exe"
copy /Y "Test_6Core_Complete.exe" "..\windows_exe\"

if %errorlevel% neq 0 (
    echo WARNING: Failed to copy executable
)

cd ..

echo.
echo === BUILD SUCCESS ===
echo Output file: build-test-6core-complete\Test_6Core_Complete.exe
echo Final location: windows_exe\Test_6Core_Complete.exe
echo.
goto end

:end