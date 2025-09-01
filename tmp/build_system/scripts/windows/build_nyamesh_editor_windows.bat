@echo off
REM ===============================================
REM NyaMesh Editor Windows Build Script
REM ===============================================
REM 
REM NyaMesh統合エディタアプリケーション Windows版ビルド:
REM - QWidget + UltraLightCoreBase 多重継承
REM - 6コア統合 Intent駆動アーキテクチャ  
REM - VSCode風モダンUI
REM - Visual Studio 2022 + Qt6 + nlohmann::json
REM 
REM 生成ファイル: NyaMeshEditor_v23.exe
REM 出力先: output/development/tests/windows/
REM 
REM Generated: 2025-07-29
REM ===============================================

echo === NyaMesh Editor v23 Windows Build ===
echo Features: 6-Core Integration, Intent-Driven UI, VSCode-style Interface
echo.

REM 環境確認
if not exist "C:\Qt\6.9.1\msvc2022_64" (
    echo ERROR: Qt6.9.1 MSVC2022 not found at C:\Qt\6.9.1\msvc2022_64
    echo Please install Qt6.9.1 with MSVC2022 compiler
    pause
    exit /b 1
)

if not exist "C:\Program Files\Microsoft Visual Studio\2022\Community" (
    echo ERROR: Visual Studio 2022 Community not found
    echo Please install Visual Studio 2022 with C++ development tools
    pause
    exit /b 1
)

REM Visual Studio 2022環境設定
echo Setting up Visual Studio 2022 environment...
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
if %errorlevel% neq 0 (
    echo ERROR: Failed to setup Visual Studio environment
    pause  
    exit /b 1
)

REM Qt6環境変数設定
echo Setting up Qt6 environment...
set CMAKE_PREFIX_PATH=C:\Qt\6.9.1\msvc2022_64
set Qt6_DIR=C:\Qt\6.9.1\msvc2022_64\lib\cmake\Qt6
set PATH=C:\Qt\6.9.1\msvc2022_64\bin;%PATH%

echo CMAKE_PREFIX_PATH: %CMAKE_PREFIX_PATH%
echo Qt6_DIR: %Qt6_DIR%

REM プロジェクトルート設定（3階層上）
set PROJECT_ROOT=%~dp0..\..\..\
echo Project Root: %PROJECT_ROOT%

REM ビルドディレクトリ作成
set BUILD_DIR=%PROJECT_ROOT%output\development\builds\windows\nyamesh-editor-windows
echo Build Directory: %BUILD_DIR%

if exist "%BUILD_DIR%" (
    echo Cleaning existing build directory...
    rmdir /s /q "%BUILD_DIR%"
)

mkdir "%BUILD_DIR%"
if %errorlevel% neq 0 (
    echo ERROR: Failed to create build directory
    pause
    exit /b 1
)

cd /d "%BUILD_DIR%"

REM CMakeファイルコピー（安全なビルド方式）
echo Copying CMake configuration...
copy /Y "%PROJECT_ROOT%build_system\cmake\targets\CMakeLists_NyaMeshEditor.txt" "CMakeLists.txt"
if %errorlevel% neq 0 (
    echo ERROR: Failed to copy CMakeLists.txt
    pause
    exit /b 1
)

REM CMake設定（NMake Makefiles使用）
echo Running CMake configuration...
cmake . -G "NMake Makefiles" ^
    -DCMAKE_BUILD_TYPE=Release ^
    -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% ^
    -DQt6_DIR=%Qt6_DIR% ^
    -DCMAKE_VERBOSE_MAKEFILE=ON

if %errorlevel% neq 0 (
    echo ERROR: CMake configuration failed
    echo.
    echo Troubleshooting:
    echo 1. Check Qt6 installation at C:\Qt\6.9.1\msvc2022_64
    echo 2. Verify Visual Studio 2022 is properly installed
    echo 3. Check internet connection for nlohmann::json download
    echo 4. Try running as Administrator
    pause
    exit /b 1
)

REM ビルド実行
echo.
echo Building NyaMesh Editor...
cmake --build . --config Release --target NyaMeshEditor --verbose

if %errorlevel% neq 0 (
    echo ERROR: Build failed
    echo.
    echo Common solutions:
    echo 1. Check for missing source files
    echo 2. Verify all cores_v23 implementations exist
    echo 3. Check for syntax errors in nyamesh_editor.cpp/h
    echo 4. Ensure nlohmann::json downloaded successfully
    pause
    exit /b 1
)

REM 成果物確認
set TARGET_EXE=NyaMeshEditor_v23.exe
if not exist "%TARGET_EXE%" (
    echo ERROR: Target executable not found: %TARGET_EXE%
    echo Build may have failed silently
    pause
    exit /b 1
)

REM 出力ディレクトリ準備
set OUTPUT_DIR=%PROJECT_ROOT%output\development\tests\windows
if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"

REM 実行ファイル・依存関係コピー
echo.
echo Deploying NyaMesh Editor...
copy /Y "%TARGET_EXE%" "%OUTPUT_DIR%\"
if %errorlevel% neq 0 (
    echo ERROR: Failed to copy executable
    pause
    exit /b 1
)

REM Qt6 DLL依存関係コピー（必要な場合）
if not exist "%OUTPUT_DIR%\Qt6Core.dll" (
    echo Copying Qt6 dependencies...
    copy /Y "C:\Qt\6.9.1\msvc2022_64\bin\Qt6Core.dll" "%OUTPUT_DIR%\"
    copy /Y "C:\Qt\6.9.1\msvc2022_64\bin\Qt6Gui.dll" "%OUTPUT_DIR%\"
    copy /Y "C:\Qt\6.9.1\msvc2022_64\bin\Qt6Widgets.dll" "%OUTPUT_DIR%\"
    
    REM platforms プラグインディレクトリ
    if not exist "%OUTPUT_DIR%\platforms" mkdir "%OUTPUT_DIR%\platforms"
    copy /Y "C:\Qt\6.9.1\msvc2022_64\plugins\platforms\qwindows.dll" "%OUTPUT_DIR%\platforms\"
)

REM 設定ファイル作成（初回起動用）
echo Creating default configuration...
echo {"theme": "auto", "fontSize": 12, "language": "auto"} > "%OUTPUT_DIR%\editor_quickaccess.json"

REM ビルド情報表示
echo.
echo === Build SUCCESS ===
echo Target: %TARGET_EXE%
echo Size: 
for %%A in ("%OUTPUT_DIR%\%TARGET_EXE%") do echo %%~zA bytes
echo Output: %OUTPUT_DIR%\%TARGET_EXE%
echo.
echo Features Included:
echo - 6-Core Integration (Editor, Settings, FileSystem, UI, Localization, QuickAccess)
echo - Intent-Driven Architecture
echo - QWidget + UltraLightCoreBase Multiple Inheritance
echo - VSCode-style Modern UI
echo - P2P Ready Architecture
echo.
echo To run: cd "%OUTPUT_DIR%" && %TARGET_EXE%
echo === Build Complete ===

REM 自動実行確認（オプション）
set /p LAUNCH_EDITOR="Launch NyaMesh Editor now? (y/N): "
if /i "%LAUNCH_EDITOR%"=="y" (
    echo Launching NyaMesh Editor...
    cd /d "%OUTPUT_DIR%"
    start "" "%TARGET_EXE%"
)

goto end

:error
echo Build failed with errors. Check the output above for details.
pause
exit /b 1

:end
echo Build script completed.
pause