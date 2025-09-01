@echo off
echo === Test_SettingsCore_v23_Integrated - Windows Build ===
echo.
echo Building SettingsCore unit test...
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
if not exist build-test-settingscore-windows mkdir build-test-settingscore-windows
cd build-test-settingscore-windows

:: Copy CMakeLists file
echo Preparing CMakeLists.txt...
copy /Y "..\CMakeLists_Test_SettingsCore.txt" CMakeLists.txt >nul

:: CMake configuration
echo Configuring CMake for SettingsCore test...
cmake . -G "NMake Makefiles" ^
    -DCMAKE_BUILD_TYPE=Release ^
    -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% ^
    -DQt6_DIR=%Qt6_DIR%

if %errorlevel% neq 0 (
    echo ERROR: CMake configuration failed
    echo Check for missing dependencies
    goto error
)

:: Build
echo Building Test_SettingsCore_v23_Integrated...
cmake --build . --config Release

if %errorlevel% neq 0 (
    echo ERROR: Build failed
    echo Check for compilation errors above
    goto error
)

:: Check executable
if not exist "Test_SettingsCore_v23_Integrated.exe" (
    echo ERROR: Executable not created
    goto error
)

:: Copy executable to windows_exe directory (primary location)
echo Copying executable to windows_exe directory...
if not exist "..\windows_exe" mkdir "..\windows_exe"
copy /Y "Test_SettingsCore_v23_Integrated.exe" "..\windows_exe\"

:: Copy required Qt6 DLLs to windows_exe
echo Copying required Qt6 DLLs to windows_exe...
copy /Y "%CMAKE_PREFIX_PATH%\bin\Qt6Core.dll" "..\windows_exe\" 2>nul
copy /Y "%CMAKE_PREFIX_PATH%\bin\Qt6Widgets.dll" "..\windows_exe\" 2>nul
copy /Y "%CMAKE_PREFIX_PATH%\bin\Qt6Gui.dll" "..\windows_exe\" 2>nul

:: Also copy to test_exe for backward compatibility
echo Copying to test_exe for backward compatibility...
if not exist "..\test_exe" mkdir "..\test_exe"
copy /Y "Test_SettingsCore_v23_Integrated.exe" "..\test_exe\"

:: Success message
echo.
echo === BUILD SUCCESS ===
echo Test_SettingsCore_v23_Integrated.exe created successfully!
echo.
echo Primary Location: windows_exe\Test_SettingsCore_v23_Integrated.exe
echo Backup Location: test_exe\Test_SettingsCore_v23_Integrated.exe
echo.
echo Run the test to verify:
echo - SettingsCore initialization
echo - Settings get/set operations
echo - File save/load functionality
echo - Intent communication
echo - Real-time change notifications
echo.
goto end

:error
echo.
echo === BUILD FAILED ===
echo Check the errors above and fix them.
cd ..
exit /b 1

:end
echo Build completed successfully!
cd ..