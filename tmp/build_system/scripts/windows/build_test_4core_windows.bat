@echo off
echo === NyaMesh Editor 4Core Integration Test Windows Build ===
echo EditorCore + SettingsCore + FileSystemCore + UICore Intent-driven Test

:: Visual Studio 2022 Environment Setup
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
if %errorlevel% neq 0 (
    echo ERROR: Visual Studio 2022 environment setup failed
    exit /b 1
)

:: Qt6 Environment Variables
set CMAKE_PREFIX_PATH=C:\Qt\6.9.1\msvc2022_64
set Qt6_DIR=C:\Qt\6.9.1\msvc2022_64\lib\cmake\Qt6

:: Create and enter build directory
if not exist build-test-4core-windows mkdir build-test-4core-windows
cd build-test-4core-windows

echo Starting CMake configuration...

:: Copy and use custom CMakeLists file
copy /Y "..\CMakeLists_Test_4Core.txt" "..\CMakeLists.txt"

:: CMake configuration (using NMake Makefiles)
cmake .. -G "NMake Makefiles" -DCMAKE_BUILD_TYPE=Release -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% -DQt6_DIR=%Qt6_DIR%

if %errorlevel% neq 0 (
    echo ERROR: CMake configuration failed
    cd ..
    exit /b 1
)

echo Starting build execution...

:: Build execution
cmake --build . --config Release --target Test_4Core_Add_UI

if %errorlevel% neq 0 (
    echo ERROR: Build failed
    cd ..
    exit /b 1
)

:: Copy executable to windows_exe directory
echo Copying executable to windows_exe directory...
if not exist "..\windows_exe" mkdir "..\windows_exe"
copy /Y "Test_4Core_Add_UI.exe" "..\windows_exe\"

if %errorlevel% neq 0 (
    echo WARNING: Failed to copy executable
)

cd ..

echo.
echo === BUILD SUCCESS ===
echo Output file: build-test-4core-windows\Test_4Core_Add_UI.exe
echo Final location: windows_exe\Test_4Core_Add_UI.exe
echo.
echo How to run:
echo cd windows_exe
echo Test_4Core_Add_UI.exe
echo.
goto end

:end