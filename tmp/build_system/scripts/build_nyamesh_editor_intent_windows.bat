@echo off
echo === NyaMesh Editor IntentConstants Integration Windows Build ===

REM Visual Studio 2022 Environment Setup
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"

REM Qt6 Environment Variables
set CMAKE_PREFIX_PATH=C:\Qt\6.9.1\msvc2022_64
set Qt6_DIR=C:\Qt\6.9.1\msvc2022_64\lib\cmake\Qt6

REM Create and Move to Build Directory
if not exist build-intent-windows mkdir build-intent-windows
cd build-intent-windows

REM CMake Configuration (Using NMake Makefiles)
cmake .. -G "NMake Makefiles" ^
    -DCMAKE_BUILD_TYPE=Release ^
    -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% ^
    -DQt6_DIR=%Qt6_DIR%

if %errorlevel% neq 0 goto error

REM Build Execution (Using Correct Target Name)
cmake --build . --config Release --target NyaMeshEditor

if %errorlevel% neq 0 goto error

REM Copy Executable File
if not exist "..\windows_exe" mkdir "..\windows_exe"
copy /Y "NyaMeshEditor_v23.exe" "..\windows_exe\"

echo === Build SUCCESS - NyaMeshEditor_v23.exe Created ===
goto end

:error
echo === Build FAILED ===
exit /b 1

:end