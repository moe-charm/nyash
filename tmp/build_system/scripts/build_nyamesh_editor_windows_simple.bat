@echo off
echo === NyaMesh Editor v23 Windows Build (Simple Version) ===

:: プロジェクトルートディレクトリを保存
set PROJECT_ROOT=%cd%
echo Current directory: %PROJECT_ROOT%

:: Visual Studio 2022環境設定
echo Setting up Visual Studio 2022 environment...
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"

:: Qt6環境変数設定
set CMAKE_PREFIX_PATH=C:\Qt\6.9.1\msvc2022_64
set Qt6_DIR=C:\Qt\6.9.1\msvc2022_64\lib\cmake\Qt6

:: ビルドディレクトリ作成・移動  
echo Creating build directory...
if not exist build mkdir build
cd build
echo Build directory: %cd%

:: CMake設定（NMake Makefiles使用）
echo Configuring with CMake...
cmake .. -G "NMake Makefiles" ^
    -DCMAKE_BUILD_TYPE=Release ^
    -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% ^
    -DQt6_DIR=%Qt6_DIR%

if %errorlevel% neq 0 (
    echo === CMake configuration FAILED ===
    cd %PROJECT_ROOT%
    exit /b 1
)

:: ビルド実行
echo Building NyaMeshEditor...
cmake --build . --config Release --target NyaMeshEditor

if %errorlevel% neq 0 (
    echo === Build FAILED ===
    cd %PROJECT_ROOT%
    exit /b 1
)

:: ビルド完了後のファイル確認
echo.
echo === Checking build output ===
if exist "NyaMeshEditor_v23.exe" (
    echo Found: NyaMeshEditor_v23.exe in %cd%
    dir NyaMeshEditor_v23.exe
) else (
    echo ERROR: NyaMeshEditor_v23.exe not found in build directory!
    cd %PROJECT_ROOT%
    exit /b 1
)

:: 実行ファイルをwindows_exeにコピー
echo.
echo === Copying to windows_exe directory ===
if not exist "%PROJECT_ROOT%\windows_exe" (
    echo Creating windows_exe directory...
    mkdir "%PROJECT_ROOT%\windows_exe"
)

:: タイムスタンプ付きのファイル名を生成
for /f "tokens=2-4 delims=/ " %%a in ('date /t') do (set mydate=%%c%%a%%b)
for /f "tokens=1-2 delims=/:" %%a in ("%time: =0%") do (set mytime=%%a%%b)

:: メインのコピー
copy /Y "NyaMeshEditor_v23.exe" "%PROJECT_ROOT%\windows_exe\NyaMeshEditor_v23_Latest.exe"
if %errorlevel% neq 0 (
    echo ERROR: Failed to copy to windows_exe directory!
) else (
    echo SUCCESS: Copied to %PROJECT_ROOT%\windows_exe\NyaMeshEditor_v23_Latest.exe
)

:: バックアップコピー（タイムスタンプ付き）
copy /Y "NyaMeshEditor_v23.exe" "%PROJECT_ROOT%\windows_exe\NyaMeshEditor_v23_%mydate%_%mytime%.exe"

:: 最終確認
echo.
echo === Final verification ===
if exist "%PROJECT_ROOT%\windows_exe\NyaMeshEditor_v23_Latest.exe" (
    echo SUCCESS: Latest build available at:
    echo %PROJECT_ROOT%\windows_exe\NyaMeshEditor_v23_Latest.exe
    dir "%PROJECT_ROOT%\windows_exe\NyaMeshEditor_v23_Latest.exe"
) else (
    echo WARNING: Could not verify the copied file!
)

:: 元のディレクトリに戻る
cd %PROJECT_ROOT%

echo.
echo === Build COMPLETED ===