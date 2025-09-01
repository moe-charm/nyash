@echo off
echo === IntentConstants Windows Build Debug ===
echo.

echo Step 1: Visual Studio 2022 Setup
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
if %errorlevel% neq 0 (
    echo ERROR: VS2022 setup failed
    goto error
)
echo VS2022 setup OK

echo.
echo Step 2: Qt6 Environment Setup
set CMAKE_PREFIX_PATH=C:\Qt\6.9.1\msvc2022_64
set Qt6_DIR=C:\Qt\6.9.1\msvc2022_64\lib\cmake\Qt6
echo CMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH%
echo Qt6_DIR=%Qt6_DIR%

echo.
echo Step 3: Build Directory Creation
if not exist build-intent-windows-debug mkdir build-intent-windows-debug
cd build-intent-windows-debug
echo Current directory: %CD%

echo.
echo Step 4: CMake Configuration
cmake .. -G "NMake Makefiles" -DCMAKE_BUILD_TYPE=Release -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% -DQt6_DIR=%Qt6_DIR%
if %errorlevel% neq 0 (
    echo ERROR: CMake configuration failed
    goto error
)
echo CMake configuration OK

echo.
echo Step 5: Build Execution
cmake --build . --config Release --target NyaMeshEditor
if %errorlevel% neq 0 (
    echo ERROR: Build failed
    goto error
)
echo Build OK

echo.
echo Step 6: File Check
dir NyaMeshEditor_v23.exe
if not exist NyaMeshEditor_v23.exe (
    echo ERROR: NyaMeshEditor_v23.exe not found
    goto error
)

echo.
echo Step 7: Copy to windows_exe
if not exist "..\windows_exe" mkdir "..\windows_exe"
copy /Y "NyaMeshEditor_v23.exe" "..\windows_exe\NyaMeshEditor_v23_IntentConstants_Debug.exe"

echo.
echo === BUILD SUCCESS ===
echo File created: NyaMeshEditor_v23_IntentConstants_Debug.exe
goto end

:error
echo.
echo === BUILD FAILED ===
exit /b 1

:end
echo.
echo === Build completed ===