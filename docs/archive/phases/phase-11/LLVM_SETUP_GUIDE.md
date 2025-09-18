# LLVM 18 セットアップガイド

Date: 2025-08-31
Platform: Linux/WSL

## 📦 LLVM 18インストール確認

```bash
$ llvm-config-18 --version
18.1.3

$ llvm-config-18 --prefix
/usr/lib/llvm-18
```

## 🔧 環境変数設定

### 方法1: シェル設定（推奨）

```bash
# ~/.bashrcまたは~/.zshrcに追加
export LLVM_SYS_180_PREFIX=/usr/lib/llvm-18

# 即座に反映
source ~/.bashrc
```

### 方法2: プロジェクトローカル設定

```bash
# プロジェクトルートに.envファイル作成
echo "LLVM_SYS_180_PREFIX=/usr/lib/llvm-18" > .env
```

### 方法3: ビルド時指定

```bash
# 環境変数を直接指定してビルド
LLVM_SYS_180_PREFIX=/usr/lib/llvm-18 cargo build --features llvm
```

## ✅ 設定確認

```bash
# 環境変数が設定されているか確認
echo $LLVM_SYS_180_PREFIX

# llvm-sysクレートのビルドテスト
cargo check --features llvm
```

## 🚀 inkwell使用例

Cargo.tomlに追加:
```toml
[dependencies]
inkwell = { version = "0.5", features = ["llvm18-0"] }

[features]
llvm = ["inkwell"]
```

テストビルド:
```bash
export LLVM_SYS_180_PREFIX=/usr/lib/llvm-18
cargo build --features llvm
```

## ⚠️ トラブルシューティング

### 問題: "could not find llvm-config"
```bash
# llvm-configへのシンボリックリンク作成
sudo ln -s /usr/bin/llvm-config-18 /usr/bin/llvm-config
```

### 問題: "LLVM_SYS_180_PREFIX not set"
```bash
# 一時的な解決
export LLVM_SYS_180_PREFIX=/usr/lib/llvm-18

# 永続的な解決（.bashrcに追加）
echo 'export LLVM_SYS_180_PREFIX=/usr/lib/llvm-18' >> ~/.bashrc
source ~/.bashrc
```

### 問題: バージョン不一致
```bash
# インストール済みLLVMバージョン確認
dpkg -l | grep llvm

# 必要に応じて正しいバージョンをインストール
sudo apt-get install llvm-18 llvm-18-dev
```

## 📋 関連ドキュメント
- [inkwell documentation](https://github.com/TheDan64/inkwell)
- [llvm-sys documentation](https://gitlab.com/taricorp/llvm-sys.rs)