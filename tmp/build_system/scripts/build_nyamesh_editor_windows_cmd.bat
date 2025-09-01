@echo off
echo === NyaMesh Editor v23 GoIntent Windows Build ===

:: Visual Studio 2022環境設定
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"

:: Qt6環境変数設定
set CMAKE_PREFIX_PATH=C:\Qt\6.9.1\msvc2022_64
set Qt6_DIR=C:\Qt\6.9.1\msvc2022_64\lib\cmake\Qt6

:: ビルドディレクトリ作成・移動
if not exist build-nyamesh-editor-windows mkdir build-nyamesh-editor-windows
cd build-nyamesh-editor-windows

:: CMake設定（NMake Makefiles使用）
cmake .. -G "NMake Makefiles" ^
    -DCMAKE_BUILD_TYPE=Release ^
    -DCMAKE_PREFIX_PATH=%CMAKE_PREFIX_PATH% ^
    -DQt6_DIR=%Qt6_DIR%

if %errorlevel% neq 0 goto error

:: ビルド実行
cmake --build . --config Release --target NyaMeshEditor

if %errorlevel% neq 0 goto error

:: 実行ファイルコピー
if not exist "..\windows_exe" mkdir "..\windows_exe"
copy /Y "NyaMeshEditor_v23.exe" "..\windows_exe\NyaMeshEditor_v23_GoIntent_Windows.exe"

echo === Build SUCCESS - NyaMeshEditor_v23_GoIntent_Windows.exe Created ===
goto end

:error
echo === Build FAILED ===
exit /b 1

:end