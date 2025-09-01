@echo off
echo === NyaMesh Editor 5Core Integration Test Windows Build (SAFE VERSION) ===
echo EditorCore + SettingsCore + FileSystemCore + UICore + LocalizationCore Intent-driven Test
echo SAFE: No root CMakeLists.txt overwrite - Build directory isolated

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
if not exist build-test-5core-windows-safe mkdir build-test-5core-windows-safe
cd build-test-5core-windows-safe

echo Starting CMake configuration (SAFE MODE - No root overwrite)...

:: SAFE: Copy SAFE CMakeLists to BUILD directory (NOT root!)
copy /Y "..\CMakeLists_Test_5Core_Safe.txt" ".\CMakeLists.txt"
if %errorlevel% neq 0 (
    echo ERROR: Failed to copy CMakeLists to build directory
    cd ..
    exit /b 1
)

:: SAFE: CMake configuration in current directory (NOT parent!)
echo Running CMake in build directory...
cmake . -G "NMake Makefiles" ^
    -DCMAKE_BUILD_TYPE=Release ^
    -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% ^
    -DQt6_DIR=%Qt6_DIR%

if %errorlevel% neq 0 (
    echo ERROR: CMake configuration failed
    cd ..
    exit /b 1
)

echo Starting build execution...

:: Build execution
cmake --build . --config Release --target Test_5Core_Add_Localization

if %errorlevel% neq 0 (
    echo ERROR: Build failed
    cd ..
    exit /b 1
)

:: Copy executable to output directory (following new structure)
echo Copying executable to output directory...
if not exist "..\output" mkdir "..\output"
if not exist "..\output\development" mkdir "..\output\development"
if not exist "..\output\development\tests" mkdir "..\output\development\tests"
if not exist "..\output\development\tests\windows" mkdir "..\output\development\tests\windows"

copy /Y "Test_5Core_Add_Localization.exe" "..\output\development\tests\windows\"

if %errorlevel% neq 0 (
    echo WARNING: Failed to copy executable to output directory
)

:: Also copy to legacy windows_exe for compatibility
if not exist "..\windows_exe" mkdir "..\windows_exe"
copy /Y "Test_5Core_Add_Localization.exe" "..\windows_exe\"

cd ..

echo.
echo === BUILD SUCCESS (SAFE VERSION) ===
echo Build directory: build-test-5core-windows-safe\Test_5Core_Add_Localization.exe
echo Output location: output\development\tests\windows\Test_5Core_Add_Localization.exe  
echo Legacy location: windows_exe\Test_5Core_Add_Localization.exe
echo.
echo SAFETY CONFIRMED: Root CMakeLists.txt was NOT modified
echo.
echo How to run:
echo cd output\development\tests\windows
echo Test_5Core_Add_Localization.exe
echo.
goto end

:end