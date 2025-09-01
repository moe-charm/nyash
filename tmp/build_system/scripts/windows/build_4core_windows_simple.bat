@echo off
echo === NyaMesh v23 4-Core Test Windows Build ===

call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"

set CMAKE_PREFIX_PATH=C:\Qt\6.9.1\msvc2022_64
set Qt6_DIR=C:\Qt\6.9.1\msvc2022_64\lib\cmake\Qt6

if not exist build-4core-test-windows mkdir build-4core-test-windows
cd build-4core-test-windows

cmake .. -G "NMake Makefiles" -DCMAKE_BUILD_TYPE=Release -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% -DQt6_DIR=%Qt6_DIR%

if %errorlevel% neq 0 (
    echo CMAKE FAILED
    cd ..
    exit /b 1
)

cmake --build . --config Release --target Test_4Core_Add_UI

if %errorlevel% neq 0 (
    echo BUILD FAILED
    cd ..
    exit /b 1
)

if not exist "..\windows_exe" mkdir "..\windows_exe"
copy /Y "Test_4Core_Add_UI.exe" "..\windows_exe\"

echo === Build SUCCESS ===
cd ..