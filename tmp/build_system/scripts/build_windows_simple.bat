@echo off
echo Building SettingsCore Windows Version...

call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
if %errorlevel% neq 0 exit /b 1

set CMAKE_PREFIX_PATH=C:\Qt\6.9.1\msvc2022_64
set Qt6_DIR=C:\Qt\6.9.1\msvc2022_64\lib\cmake\Qt6

if not exist build-windows-real mkdir build-windows-real
cd build-windows-real

cmake .. -G "NMake Makefiles" -DCMAKE_BUILD_TYPE=Release -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% -DQt6_DIR=%Qt6_DIR%
if %errorlevel% neq 0 exit /b 1

nmake
if %errorlevel% neq 0 exit /b 1

copy NyaMeshEditor_v23.exe ..\windows_exe\NyaMeshEditor_v23_SettingsCore_Real.exe

echo Build completed successfully