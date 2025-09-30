# Nyash Test Applications (Compiled Binaries)

このディレクトリには、Nyashのテストアプリケーションのコンパイル済みバイナリが格納されています。

## 🚀 実行方法

```bash
# 直接実行
./artifacts/apps/app

# または Makefile 経由（推奨）
make run-app APP=app
```

## 📋 アプリケーション一覧

### 基本テスト
- **app** - 基本テストアプリ (3.3MB)
- **app_empty** - 空のアプリケーション
- **app_len** - 長さ計算テスト

### AST/Literal テスト
- **app_alit** - AST Literal 基本テスト (12MB)
- **app_alit_print** - AST Literal 出力テスト
- **app_alit_verbose** - AST Literal 詳細出力

### ループテスト
- **app_loop** - 基本ループテスト
- **app_loop2** - ループテスト2
- **app_loop_cf** - ループ制御フローテスト
- **app_loop_vmap** - ループ変数マッピング

### LLVM バックエンドテスト
- **app_echo_llvm** - LLVM エコーテスト
- **app_llvm_guide** - LLVM ガイド
- **app_llvm_test** - LLVM 基本テスト
- **app_llvmlite_esc** - llvmlite エスケープテスト
- **app_ll_esc_fix** - LLVM エスケープ修正
- **app_ll_verify** - LLVM 検証

### 依存関係テスト
- **app_dep_tree_py** - Python依存関係ツリー
- **app_dep_tree_rust** - Rust依存関係ツリー

### その他
- **app_async** - 非同期処理テスト
- **app_gc_smoke** - GCスモークテスト
- **app_link** - リンクテスト

## 🔧 ビルド方法

```bash
# 全アプリビルド
cargo build --release --bin app
cargo build --release --bin app_alit
# ... (各アプリケーション)

# または Makefile 経由
make build-all-apps
```

## 📊 容量情報

- **合計サイズ**: 625MB
- **ファイル数**: 60個
- **形式**: ELF 64-bit executable (デバッグ情報付き)

## ⚠️ 注意事項

- このディレクトリは `.gitignore` で除外されています
- デバッグ情報付きのため、ファイルサイズが大きくなっています
- Release ビルドで再生成することで容量削減可能

## 🎯 開発時のTips

### よく使うアプリへのエイリアス設定

```bash
# ~/.bashrc または ~/.zshrc に追加
alias nyash-app='./artifacts/apps/app'
alias nyash-loop='./artifacts/apps/app_loop'
alias nyash-llvm='./artifacts/apps/app_llvm_test'
```

### ルートからの実行（従来の方法）

従来はルートにシンボリックリンクがありましたが、
整理のため削除されました。必要な場合は：

```bash
# 一時的なリンク作成
ln -s artifacts/apps/app ./app

# 使用後削除
rm ./app
```

## 📝 更新履歴

- **2025-09-29**: ChatGPT により一括ビルド・配置
- **2025-09-30**: ルートのシンボリックリンク整理、README 追加

---

**生成日**: 2025-09-30
**管理**: Nyash開発チーム