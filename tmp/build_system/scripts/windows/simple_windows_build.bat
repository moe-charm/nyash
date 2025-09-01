@echo off
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
set CMAKE_PREFIX_PATH=C:\Qt\6.9.1\msvc2022_64
cd build-windows-v23
cmake .. -G "Visual Studio 17 2022" -A x64
cmake --build . --config Release
if %errorlevel% neq 0 goto error

echo Copying to C:\git\moe-charm\nyamesh_editor\windows_exe...
if not exist "C:\git\moe-charm\nyamesh_editor\windows_exe" mkdir "C:\git\moe-charm\nyamesh_editor\windows_exe"
copy /Y "bin\Release\NyaMeshV23WindowsTest.exe" "C:\git\moe-charm\nyamesh_editor\windows_exe\"

echo Copying Qt DLLs...
set QT_DLL_SOURCE=C:\git\moe-charm\CharmCode_Editor\windows-executables
if exist "%QT_DLL_SOURCE%\Qt6Core.dll" (
    copy /Y "%QT_DLL_SOURCE%\Qt6Core.dll" "C:\git\moe-charm\nyamesh_editor\windows_exe\"
    copy /Y "%QT_DLL_SOURCE%\Qt6Gui.dll" "C:\git\moe-charm\nyamesh_editor\windows_exe\"
    copy /Y "%QT_DLL_SOURCE%\Qt6Widgets.dll" "C:\git\moe-charm\nyamesh_editor\windows_exe\"
    
    echo Copying Qt platform plugin...
    if not exist "C:\git\moe-charm\nyamesh_editor\windows_exe\platforms" mkdir "C:\git\moe-charm\nyamesh_editor\windows_exe\platforms"
    copy /Y "%QT_DLL_SOURCE%\platforms\qwindows.dll" "C:\git\moe-charm\nyamesh_editor\windows_exe\platforms\"
    
    echo Qt DLLs and plugins copied!
) else (
    echo Warning: Qt DLLs not found in %QT_DLL_SOURCE%
)
echo Copy complete!
goto end

:error
echo Build failed!
exit /b 1

:end
echo Build complete!
pause