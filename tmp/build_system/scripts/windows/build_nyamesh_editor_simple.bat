@echo off
REM Simple NyaMesh Editor Windows Build Script (English only)
echo === NyaMesh Editor v23 Windows Build ===

REM Environment checks
if not exist "C:\Qt\6.9.1\msvc2022_64" (
    echo ERROR: Qt6.9.1 MSVC2022 not found
    exit /b 1
)

if not exist "C:\Program Files\Microsoft Visual Studio\2022\Community" (
    echo ERROR: Visual Studio 2022 Community not found
    exit /b 1
)

REM Setup Visual Studio 2022
echo Setting up Visual Studio 2022 environment...
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
if %errorlevel% neq 0 exit /b 1

REM Setup Qt6 environment
echo Setting up Qt6 environment...
set CMAKE_PREFIX_PATH=C:\Qt\6.9.1\msvc2022_64
set Qt6_DIR=C:\Qt\6.9.1\msvc2022_64\lib\cmake\Qt6
set PATH=C:\Qt\6.9.1\msvc2022_64\bin;%PATH%

REM Project root setup
set PROJECT_ROOT=%~dp0..\..\..\
echo Project Root: %PROJECT_ROOT%

REM Build directory setup
set BUILD_DIR=%PROJECT_ROOT%output\development\builds\windows\nyamesh-editor-simple
echo Build Directory: %BUILD_DIR%

if exist "%BUILD_DIR%" rmdir /s /q "%BUILD_DIR%"
mkdir "%BUILD_DIR%"
cd /d "%BUILD_DIR%"

REM Copy CMake configuration
echo Copying CMake configuration...
copy /Y "%PROJECT_ROOT%build_system\cmake\targets\CMakeLists_NyaMeshEditor.txt" "CMakeLists.txt"
if %errorlevel% neq 0 exit /b 1

REM CMake configuration
echo Running CMake configuration...
cmake . -G "NMake Makefiles" ^
    -DCMAKE_BUILD_TYPE=Release ^
    -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% ^
    -DQt6_DIR=%Qt6_DIR%

if %errorlevel% neq 0 (
    echo ERROR: CMake configuration failed
    exit /b 1
)

REM Build execution
echo Building NyaMesh Editor...
cmake --build . --config Release --target NyaMeshEditor

if %errorlevel% neq 0 (
    echo ERROR: Build failed
    exit /b 1
)

REM Check target executable
set TARGET_EXE=NyaMeshEditor_v23.exe
if not exist "%TARGET_EXE%" (
    echo ERROR: Target executable not found
    exit /b 1
)

REM Deploy to output directory
set OUTPUT_DIR=%PROJECT_ROOT%output\development\tests\windows
if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"

echo Deploying NyaMesh Editor...
copy /Y "%TARGET_EXE%" "%OUTPUT_DIR%\"
if %errorlevel% neq 0 exit /b 1

REM Copy Qt6 dependencies
if not exist "%OUTPUT_DIR%\Qt6Core.dll" (
    echo Copying Qt6 dependencies...
    copy /Y "C:\Qt\6.9.1\msvc2022_64\bin\Qt6Core.dll" "%OUTPUT_DIR%\"
    copy /Y "C:\Qt\6.9.1\msvc2022_64\bin\Qt6Gui.dll" "%OUTPUT_DIR%\"
    copy /Y "C:\Qt\6.9.1\msvc2022_64\bin\Qt6Widgets.dll" "%OUTPUT_DIR%\"
    
    if not exist "%OUTPUT_DIR%\platforms" mkdir "%OUTPUT_DIR%\platforms"
    copy /Y "C:\Qt\6.9.1\msvc2022_64\plugins\platforms\qwindows.dll" "%OUTPUT_DIR%\platforms\"
)

REM Create default configuration
echo {"theme": "auto", "fontSize": 12, "language": "auto"} > "%OUTPUT_DIR%\editor_quickaccess.json"

REM Display build info
echo.
echo === Build SUCCESS ===
echo Target: %TARGET_EXE%
for %%A in ("%OUTPUT_DIR%\%TARGET_EXE%") do echo Size: %%~zA bytes
echo Output: %OUTPUT_DIR%\%TARGET_EXE%
echo === Build Complete ===

pause