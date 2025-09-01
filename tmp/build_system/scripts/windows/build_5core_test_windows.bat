@echo off
echo === NyaMesh v23 5-Core Test Windows Build ===
echo Building 5-Core Integration Test (LocalizationCore excluded)

:: Visual Studio 2022 environment
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"

:: Qt6 environment
set CMAKE_PREFIX_PATH=C:\Qt\6.9.1\msvc2022_64
set Qt6_DIR=C:\Qt\6.9.1\msvc2022_64\lib\cmake\Qt6

:: Create build directory
if not exist build-5core-test-windows mkdir build-5core-test-windows
cd build-5core-test-windows

:: Configure with CMake
echo Configuring CMake...
cmake .. -G "NMake Makefiles" ^
    -DCMAKE_BUILD_TYPE=Release ^
    -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% ^
    -DQt6_DIR=%Qt6_DIR%

if %errorlevel% neq 0 goto error

:: Build 5-Core test
echo Building Test5Core_v23...
cmake --build . --config Release --target Test5Core_v23

if %errorlevel% neq 0 goto error

:: Copy to windows_exe
echo Copying to windows_exe...
if not exist "..\windows_exe" mkdir "..\windows_exe"
copy /Y "Test5Core_v23.exe" "..\windows_exe\"

:: Copy Qt DLLs from CharmCode_Editor
echo Copying Qt DLLs...
set QT_DLL_SOURCE=C:\git\moe-charm\CharmCode_Editor\windows-executables
if exist "%QT_DLL_SOURCE%\Qt6Core.dll" (
    copy /Y "%QT_DLL_SOURCE%\Qt6Core.dll" "..\windows_exe\"
    copy /Y "%QT_DLL_SOURCE%\Qt6Gui.dll" "..\windows_exe\"
    copy /Y "%QT_DLL_SOURCE%\Qt6Widgets.dll" "..\windows_exe\"
    
    :: Copy platform plugin
    if not exist "..\windows_exe\platforms" mkdir "..\windows_exe\platforms"
    copy /Y "%QT_DLL_SOURCE%\platforms\qwindows.dll" "..\windows_exe\platforms\"
    
    echo Qt DLLs copied successfully!
) else (
    echo Warning: Qt DLLs not found in %QT_DLL_SOURCE%
)

echo === Build SUCCESS - 5-Core Test Ready ===
echo Executable: windows_exe\Test5Core_v23.exe
echo Complexity Reductions:
echo - EditorCore: 82.7%
echo - UICoordinator: 86.7%
echo - QuickAccessCore: 89.8%
goto end

:error
echo === Build FAILED ===
exit /b 1

:end
cd ..
echo Build complete!
pause