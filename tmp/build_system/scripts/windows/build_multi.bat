@echo off
echo === NyaMesh Editor v23 Multi-Core Build (Windows) ===
echo.

:: Visual Studio 2022環境設定
echo [1/4] Setting up Visual Studio 2022 environment...
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
if %errorlevel% neq 0 (
    echo ERROR: Visual Studio 2022 not found!
    exit /b 1
)

:: Qt6環境変数設定
echo [2/4] Setting up Qt6 environment...
set CMAKE_PREFIX_PATH=C:\Qt\6.9.1\msvc2022_64
set Qt6_DIR=C:\Qt\6.9.1\msvc2022_64\lib\cmake\Qt6

:: ビルドディレクトリ作成・移動
echo [3/4] Creating build directory...
if not exist build mkdir build
cd build

:: CMake設定（NMake Makefiles使用）
echo [4/4] Configuring and building...
cmake .. -G "NMake Makefiles" ^
    -DCMAKE_BUILD_TYPE=Release ^
    -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% ^
    -DQt6_DIR=%Qt6_DIR%

if %errorlevel% neq 0 (
    echo ERROR: CMake configuration failed!
    exit /b 1
)

:: ビルド実行
cmake --build . --config Release

if %errorlevel% neq 0 (
    echo ERROR: Build failed!
    exit /b 1
)

:: 実行ファイルコピー
echo.
echo Copying executable to output directory...
if not exist "..\output\development\tests\windows" mkdir "..\output\development\tests\windows"
copy /Y "NyaMeshEditor_v23.exe" "..\output\development\tests\windows\"

echo.
echo === Build SUCCESS ===
echo Executable: output\development\tests\windows\NyaMeshEditor_v23.exe
cd ..