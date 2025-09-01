@echo off
echo === NyaMesh Editor 6-Core v23 Integrated - Windows Build ===
echo.
echo Building revolutionary 6-core integrated text editor...
echo Features:
echo - EditorCore_v23_Integrated: Advanced text editing with auto-save
echo - SettingsCore_v23_Integrated: Dynamic configuration management  
echo - FileSystemCore_v23_Integrated: File operations with monitoring
echo - UICore_v23_Integrated: Theme management and responsive design
echo - LocalizationCore_v23_Integrated: Multi-language support
echo - QuickAccessCore_v23_Integrated: Bookmarks and recent files
echo - P2P Transport Layer: Inter-core communication
echo.

:: Visual Studio 2022 environment setup
echo Setting up Visual Studio 2022 environment...
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
if %errorlevel% neq 0 (
    echo ERROR: Failed to setup Visual Studio environment
    echo Make sure Visual Studio 2022 is installed
    goto error
)

:: Qt6 environment variables
echo Setting up Qt6 environment...
set CMAKE_PREFIX_PATH=C:\Qt\6.9.1\msvc2022_64
set Qt6_DIR=C:\Qt\6.9.1\msvc2022_64\lib\cmake\Qt6

if not exist "%CMAKE_PREFIX_PATH%" (
    echo ERROR: Qt6 not found at %CMAKE_PREFIX_PATH%
    echo Please install Qt6.9.1 for MSVC2022_64
    goto error
)

:: Create build directory
echo Creating build directory...
if not exist build-6core-integrated-windows mkdir build-6core-integrated-windows
cd build-6core-integrated-windows

:: Copy CMakeLists file with correct name
echo Preparing CMakeLists.txt...
copy /Y ..\CMakeLists_6Core_Integrated.txt CMakeLists.txt >nul

:: CMake configuration
echo Configuring CMake for 6-Core Integrated build...
cmake . -G "NMake Makefiles" ^
    -DCMAKE_BUILD_TYPE=Release ^
    -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% ^
    -DQt6_DIR=%Qt6_DIR%

if %errorlevel% neq 0 (
    echo ERROR: CMake configuration failed
    echo Check Qt6 installation and paths
    goto error
)

:: Build
echo Building NyaMeshEditor_6Core_v23_Integrated...
cmake --build . --config Release --target NyaMeshEditor_6Core_v23_Integrated

if %errorlevel% neq 0 (
    echo ERROR: Build failed
    echo Check for compilation errors above
    goto error
)

:: Check executable
if not exist "NyaMeshEditor_6Core_v23_Integrated.exe" (
    echo ERROR: Executable not created
    goto error
)

:: Copy executable
echo Copying executable to output directory...
if not exist "..\windows_exe" mkdir "..\windows_exe"
copy /Y "NyaMeshEditor_6Core_v23_Integrated.exe" "..\windows_exe\"

:: Copy required Qt6 DLLs
echo Copying required Qt6 DLLs...
copy /Y "%CMAKE_PREFIX_PATH%\bin\Qt6Core.dll" "..\windows_exe\" 2>nul
copy /Y "%CMAKE_PREFIX_PATH%\bin\Qt6Widgets.dll" "..\windows_exe\" 2>nul
copy /Y "%CMAKE_PREFIX_PATH%\bin\Qt6Gui.dll" "..\windows_exe\" 2>nul

:: Success message
echo.
echo === BUILD SUCCESS ===
echo NyaMeshEditor_6Core_v23_Integrated.exe created successfully!
echo.
echo Output location: windows_exe\NyaMeshEditor_6Core_v23_Integrated.exe
echo.
echo Features included:
echo - 6 Integrated Cores with P2P communication
echo - UltraLightCoreBase architecture
echo - Modern Qt6 UI with theme support
echo - Multi-language support
echo - Advanced file management
echo - Quick access and bookmarks
echo - Real-time inter-core synchronization
echo.
echo Ready to run: cd windows_exe and NyaMeshEditor_6Core_v23_Integrated.exe
goto end

:error
echo.
echo === BUILD FAILED ===
echo Check the errors above and fix them.
echo.
echo Common issues:
echo - Visual Studio 2022 not installed
echo - Qt6.9.1 MSVC2022_64 not installed at C:\Qt\6.9.1\msvc2022_64
echo - Internet connection required for nlohmann/json
echo - Ensure all source files are present
cd ..
exit /b 1

:end
echo.
echo Build completed. Check windows_exe directory for the executable.
cd ..