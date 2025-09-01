@echo off
echo === Simple Qt Test Build ===

REM Visual Studio 2022 setup
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
if %errorlevel% neq 0 exit /b 1

REM Qt6 environment
set CMAKE_PREFIX_PATH=C:\Qt\6.9.1\msvc2022_64
set Qt6_DIR=C:\Qt\6.9.1\msvc2022_64\lib\cmake\Qt6

REM Build directory
set PROJECT_ROOT=%~dp0..\..\..\
set BUILD_DIR=%PROJECT_ROOT%output\development\builds\windows\simple-test
if exist "%BUILD_DIR%" rmdir /s /q "%BUILD_DIR%"
mkdir "%BUILD_DIR%"
cd /d "%BUILD_DIR%"

REM Copy CMake file
copy /Y "%PROJECT_ROOT%build_system\cmake\targets\CMakeLists_SimpleTest.txt" "CMakeLists.txt"

REM Build
cmake . -G "NMake Makefiles" -DCMAKE_BUILD_TYPE=Debug -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% -DQt6_DIR=%Qt6_DIR%
if %errorlevel% neq 0 exit /b 1

cmake --build . --config Debug --target SimpleQtTest
if %errorlevel% neq 0 exit /b 1

REM Deploy
set OUTPUT_DIR=%PROJECT_ROOT%output\development\tests\windows
copy /Y "SimpleQtTest.exe" "%OUTPUT_DIR%\"

echo === Simple Test Build SUCCESS ===
pause