# LLVM 18 Windows セットアップガイド

Date: 2025-08-31
Platform: Windows

## 📦 インストール方法

### 方法1: 公式インストーラー（推奨）

1. **LLVM公式サイトからダウンロード**
   - https://github.com/llvm/llvm-project/releases
   - `LLVM-18.1.8-win64.exe` をダウンロード（または最新の18.x版）

2. **インストーラー実行**
   - 管理者権限で実行
   - インストール先: `C:\Program Files\LLVM` （デフォルト推奨）
   - **重要**: "Add LLVM to the system PATH" にチェック！

3. **環境変数設定**
   ```powershell
   # PowerShell（管理者権限）で実行
   [Environment]::SetEnvironmentVariable("LLVM_SYS_180_PREFIX", "C:\Program Files\LLVM", "User")
   ```

### 方法2: Chocolatey（パッケージマネージャー）

```powershell
# 管理者権限のPowerShellで実行
# Chocolateyインストール（未インストールの場合）
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# LLVM 18インストール
choco install llvm --version=18.1.8

# 環境変数設定
[Environment]::SetEnvironmentVariable("LLVM_SYS_180_PREFIX", "C:\ProgramData\chocolatey\lib\llvm\tools", "User")
```

### 方法3: winget（Windows Package Manager）

```powershell
# PowerShellで実行
winget install LLVM.LLVM --version 18.1.8

# 環境変数設定（インストール先確認後）
[Environment]::SetEnvironmentVariable("LLVM_SYS_180_PREFIX", "C:\Program Files\LLVM", "User")
```

## 🔧 環境変数設定（GUI経由）

1. **システムのプロパティを開く**
   - Win + X → システム → システムの詳細設定
   - または「sysdm.cpl」を実行

2. **環境変数を設定**
   - 「環境変数」ボタンをクリック
   - ユーザー環境変数で「新規」
   - 変数名: `LLVM_SYS_180_PREFIX`
   - 変数値: `C:\Program Files\LLVM`

3. **PATH確認**
   - `C:\Program Files\LLVM\bin` がPATHに含まれていることを確認

## ✅ インストール確認

```powershell
# PowerShellで実行
# LLVMバージョン確認
llvm-config --version

# 環境変数確認
echo $env:LLVM_SYS_180_PREFIX

# または cmd.exe で
echo %LLVM_SYS_180_PREFIX%
```

## 🚀 Visual Studio依存関係

WindowsでLLVMを使う場合、Visual Studioのビルドツールが必要：

### Visual Studio Build Tools（最小構成）
```powershell
# wingetでインストール
winget install Microsoft.VisualStudio.2022.BuildTools

# または直接ダウンロード
# https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
```

必要なコンポーネント：
- MSVC v143 - VS 2022 C++ x64/x86 ビルドツール
- Windows 11 SDK（または Windows 10 SDK）

## 🔨 Rustプロジェクトでの使用

1. **Cargo.tomlに追加**
```toml
[dependencies]
inkwell = { version = "0.5", features = ["llvm18-0"] }

[features]
llvm = ["inkwell"]
```

2. **ビルド実行**
```powershell
# PowerShellで実行
$env:LLVM_SYS_180_PREFIX="C:\Program Files\LLVM"
cargo build --features llvm

# または永続的に設定後
cargo build --features llvm
```

## ⚠️ トラブルシューティング

### 問題: "llvm-config not found"
```powershell
# PATHに追加されているか確認
$env:Path -split ';' | Select-String "LLVM"

# 手動でPATHに追加
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\Program Files\LLVM\bin", "User")
```

### 問題: "LINK : fatal error LNK1181"
- Visual Studio Build Toolsがインストールされているか確認
- 必要に応じて再起動

### 問題: バージョン不一致
```powershell
# インストール済みLLVMを確認
llvm-config --version

# 古いバージョンをアンインストール
# コントロールパネル → プログラムと機能 → LLVM
```

## 🎯 クイックセットアップ（コピペ用）

```powershell
# 管理者権限のPowerShellで実行
# 1. Chocolateyインストール（未インストールの場合）
Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# 2. LLVM 18インストール
choco install llvm --version=18.1.8 -y

# 3. 環境変数設定
[Environment]::SetEnvironmentVariable("LLVM_SYS_180_PREFIX", "C:\ProgramData\chocolatey\lib\llvm\tools", "User")

# 4. 新しいPowerShellウィンドウを開いて確認
llvm-config --version
echo $env:LLVM_SYS_180_PREFIX
```

## 📋 関連リンク
- [LLVM Releases](https://github.com/llvm/llvm-project/releases)
- [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
- [Chocolatey](https://chocolatey.org/)
- [Windows Package Manager](https://github.com/microsoft/winget-cli)