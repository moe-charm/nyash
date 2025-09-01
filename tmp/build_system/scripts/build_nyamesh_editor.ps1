Write-Host "=== NyaMesh Editor v23 GoIntent Windows Build ==="

# Visual Studio 2022環境設定
& "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"

# Qt6環境変数設定
$env:CMAKE_PREFIX_PATH = "C:\Qt\6.9.1\msvc2022_64"
$env:Qt6_DIR = "C:\Qt\6.9.1\msvc2022_64\lib\cmake\Qt6"

# ビルドディレクトリ作成・移動
if (!(Test-Path "build-nyamesh-editor-windows")) {
    New-Item -ItemType Directory -Path "build-nyamesh-editor-windows"
}
Set-Location "build-nyamesh-editor-windows"

# CMake設定
cmake .. -G "NMake Makefiles" -DCMAKE_BUILD_TYPE=Release -DCMAKE_PREFIX_PATH="$env:CMAKE_PREFIX_PATH" -DQt6_DIR="$env:Qt6_DIR"

if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: CMake configuration failed"
    exit 1
}

# ビルド実行
cmake --build . --config Release --target NyaMeshEditor

if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Build failed"
    exit 1
}

# 実行ファイルコピー
if (!(Test-Path "..\windows_exe")) {
    New-Item -ItemType Directory -Path "..\windows_exe"
}
Copy-Item "NyaMeshEditor_v23.exe" "..\windows_exe\NyaMeshEditor_v23_GoIntent_Windows.exe"

Write-Host "=== Build SUCCESS - NyaMeshEditor_v23_GoIntent_Windows.exe Created ==="