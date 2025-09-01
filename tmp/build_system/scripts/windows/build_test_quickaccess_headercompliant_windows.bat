@echo off
echo === QuickAccessCore v23 Header Compliant Unit Test Windows Build ===
echo ヘッダー完全準拠実装の動作確認ビルド
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
if not exist build-test-quickaccess-headercompliant-windows mkdir build-test-quickaccess-headercompliant-windows
cd build-test-quickaccess-headercompliant-windows

echo Starting CMake configuration (SAFE MODE - No root overwrite)...

:: SAFE: Copy SAFE CMakeLists to BUILD directory (NOT root!)
copy /Y "..\CMakeLists_Test_QuickAccessCore_HeaderCompliant.txt" ".\CMakeLists.txt"
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
cmake --build . --config Release --target Test_QuickAccessCore_HeaderCompliant

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

copy /Y "Test_QuickAccessCore_HeaderCompliant.exe" "..\output\development\tests\windows\"

if %errorlevel% neq 0 (
    echo WARNING: Failed to copy executable to output directory
)

:: Also copy to legacy windows_exe for compatibility
if not exist "..\windows_exe" mkdir "..\windows_exe"
copy /Y "Test_QuickAccessCore_HeaderCompliant.exe" "..\windows_exe\"

:: Auto-copy to C:\git\moe-charm\nyamesh_editor\test_exe for user convenience
echo Copying to C:\git\moe-charm\nyamesh_editor\test_exe for convenience...
if not exist "C:\git\moe-charm\nyamesh_editor\test_exe" mkdir "C:\git\moe-charm\nyamesh_editor\test_exe"
copy /Y "Test_QuickAccessCore_HeaderCompliant.exe" "C:\git\moe-charm\nyamesh_editor\test_exe\"

if %errorlevel% neq 0 (
    echo WARNING: Failed to copy executable to convenience directory
) else (
    echo SUCCESS: Executable copied to C:\git\moe-charm\nyamesh_editor\test_exe\Test_QuickAccessCore_HeaderCompliant.exe
)

cd ..

echo.
echo === BUILD SUCCESS (SAFE VERSION) - QuickAccessCore HEADER COMPLIANT UNIT TEST ===
echo Build directory: build-test-quickaccess-headercompliant-windows\Test_QuickAccessCore_HeaderCompliant.exe
echo Output location: output\development\tests\windows\Test_QuickAccessCore_HeaderCompliant.exe  
echo Legacy location: windows_exe\Test_QuickAccessCore_HeaderCompliant.exe
echo Convenience location: C:\git\moe-charm\nyamesh_editor\test_exe\Test_QuickAccessCore_HeaderCompliant.exe
echo.
echo SAFETY CONFIRMED: Root CMakeLists.txt was NOT modified
echo.
echo QuickAccessCore Header Compliant Implementation Test:
echo   - QuickAccessCore_v23_Integrated_HeaderCompliant.cpp verification
echo   - Complete header definition compliance
echo   - Intent-driven architecture validation
echo   - LocalizationCore success pattern replication
echo.
echo How to run (multiple locations):
echo   cd output\development\tests\windows ^&^& Test_QuickAccessCore_HeaderCompliant.exe
echo   cd windows_exe ^&^& Test_QuickAccessCore_HeaderCompliant.exe  
echo   C:\git\moe-charm\nyamesh_editor\test_exe\Test_QuickAccessCore_HeaderCompliant.exe
echo.
goto end

:end