@echo off
echo === NyaMesh Editor 7Core P2P Ready Integration Test Windows Build ===
echo EditorCore + SettingsCore + FileSystemCore + UICore + LocalizationCore + QuickAccessCore + AppNyaMeshNode (7 cores P2P Ready, Composition Pattern)
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
if not exist build-test-7core-p2p-ready-windows mkdir build-test-7core-p2p-ready-windows
cd build-test-7core-p2p-ready-windows

echo Starting CMake configuration (SAFE MODE - No root overwrite)...

:: SAFE: Copy SAFE CMakeLists to BUILD directory (NOT root!)
copy /Y "..\CMakeLists_Test_7Core_P2P_Ready.txt" ".\CMakeLists.txt"
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
cmake --build . --config Release --target Test_7Core_P2P_Ready

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

copy /Y "Test_7Core_P2P_Ready.exe" "..\output\development\tests\windows\"

if %errorlevel% neq 0 (
    echo WARNING: Failed to copy executable to output directory
)

:: Also copy to legacy windows_exe for compatibility
if not exist "..\windows_exe" mkdir "..\windows_exe"
copy /Y "Test_7Core_P2P_Ready.exe" "..\windows_exe\"

:: Auto-copy to C:\git\moe-charm\nyamesh_editor\test_exe for user convenience
echo Copying to C:\git\moe-charm\nyamesh_editor\test_exe for convenience...
if not exist "C:\git\moe-charm\nyamesh_editor\test_exe" mkdir "C:\git\moe-charm\nyamesh_editor\test_exe"
copy /Y "Test_7Core_P2P_Ready.exe" "C:\git\moe-charm\nyamesh_editor\test_exe\"

if %errorlevel% neq 0 (
    echo WARNING: Failed to copy executable to convenience directory
) else (
    echo SUCCESS: Executable copied to C:\git\moe-charm\nyamesh_editor\test_exe\Test_7Core_P2P_Ready.exe
)

cd ..

echo.
echo === BUILD SUCCESS (SAFE VERSION) - 7 CORES P2P READY INTEGRATION ===
echo Build directory: build-test-7core-p2p-ready-windows\Test_7Core_P2P_Ready.exe
echo Output location: output\development\tests\windows\Test_7Core_P2P_Ready.exe
echo Legacy location: windows_exe\Test_7Core_P2P_Ready.exe
echo Convenience location: C:\git\moe-charm\nyamesh_editor\test_exe\Test_7Core_P2P_Ready.exe
echo.
echo SAFETY CONFIRMED: Root CMakeLists.txt was NOT modified
echo.
echo 7-Core P2P Ready Integration Success:
echo   1. EditorCore_v23_Integrated - Text editing and buffer management
echo   2. SettingsCore_v23_Integrated - Settings management and persistence
echo   3. FileSystemCore_v23_Integrated - File operations and directory management
echo   4. UICore_v23_Integrated - UI control and theme management
echo   5. LocalizationCore_v23_Integrated - Multi-language support and translation
echo   6. QuickAccessCore_v23_Integrated - Quick access, bookmarks, and history (Header Compliant)
echo   7. AppNyaMeshNode - Application Integration, P2P Communication, Qt Bridge (Composition Pattern)
echo.
echo ARCHITECTURE ACHIEVEMENT: Gemini Composition + Bridge Pattern Completed!
echo   - AppNyaMeshNode: QObject + UltraLightCoreBase multiple inheritance
echo   - Test7CoreP2PReadyApp: Pure Qt Widget + signals/slots communication
echo   - UI <-> NyaMesh P2P Bridge: Intent sending/receiving via Qt signals
echo   - P2P Ready: Can connect to other editor instances in the future
echo   - Gradual Intent Migration: Phase 1 (AppNode integration) -> Phase 2 (Intent-driven UI)
echo.
echo How to run (multiple locations):
echo   cd output\development\tests\windows ^&^& Test_7Core_P2P_Ready.exe
echo   cd windows_exe ^&^& Test_7Core_P2P_Ready.exe
echo   C:\git\moe-charm\nyamesh_editor\test_exe\Test_7Core_P2P_Ready.exe
echo.
goto end

:end