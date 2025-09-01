@echo off
echo === NyaMesh Editor 6Core Complete Integration Test Windows Build - Intent Driven ===
echo EditorCore + SettingsCore + FileSystemCore + UICore + LocalizationCore + QuickAccessCore Intent-driven Test

:: Visual Studio 2022環境設定
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"

:: Qt6環境変数設定
set CMAKE_PREFIX_PATH=C:\Qt\6.9.1\msvc2022_64
set Qt6_DIR=C:\Qt\6.9.1\msvc2022_64\lib\cmake\Qt6

:: ビルドディレクトリ作成・移動
if not exist build-test-6core-intent-windows mkdir build-test-6core-intent-windows
cd build-test-6core-intent-windows

:: CMakeファイルをコピー
copy /Y "..\CMakeLists_Test_6Core_Intent.txt" ".\CMakeLists.txt"

if %errorlevel% neq 0 (
    echo CMakeLists.txt copy failed
    goto error
)

:: CMake設定（NMake Makefiles使用）
echo Starting CMake configuration...
cmake . -G "NMake Makefiles" ^
    -DCMAKE_BUILD_TYPE=Release ^
    -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% ^
    -DQt6_DIR=%Qt6_DIR%

if %errorlevel% neq 0 goto error

:: ビルド実行
echo Starting build execution...
cmake --build . --config Release --target Test_6Core_Complete_Intent

if %errorlevel% neq 0 goto error

:: 実行ファイルコピー
if not exist "..\windows_exe" mkdir "..\windows_exe"
copy /Y "Test_6Core_Complete_Intent.exe" "..\windows_exe\"

echo.
echo === BUILD SUCCESS ===
echo Output file: build-test-6core-intent-windows\Test_6Core_Complete_Intent.exe
echo Final location: windows_exe\Test_6Core_Complete_Intent.exe
echo.
echo How to run:
echo cd windows_exe
echo Test_6Core_Complete_Intent.exe

goto end

:error
echo === Build FAILED ===
exit /b 1

:end