[Archived] 旧10.1系ドキュメントです。最新は ../INDEX.md を参照してください。

# Phase 10.1b - 環境設定とセットアップ

## 🎯 このフェーズの目的
PythonParserBox実装に必要な開発環境を整える。

## 📋 セットアップ手順

### 1. Python 3.11環境の固定
```bash
# pyenvを使用する場合
pyenv install 3.11.9
pyenv local 3.11.9

# または直接指定
python3.11 --version  # 3.11.9であることを確認
```

### 2. Cargo.tomlへの依存関係追加
```toml
[dependencies]
pyo3 = { version = "0.22", features = ["auto-initialize"] }
pyo3-numpy = "0.22"  # NumPy連携用（Phase 3で使用）
serde_json = "1.0"   # JSON中間表現用
```

### 3. 環境変数の設定
```bash
# テレメトリー用
export NYASH_PYTHONPARSER_TELEMETRY=1  # 基本統計
export NYASH_PYTHONPARSER_TELEMETRY=2  # 詳細ログ
export NYASH_PYTHONPARSER_STRICT=1     # フォールバック時にパニック（CI用）
```

### 4. ディレクトリ構造の準備
```
src/boxes/python_parser_box/
├── mod.rs              # メインモジュール
├── py_helper.rs        # Python側ヘルパー
├── converter.rs        # AST変換器
└── telemetry.rs        # テレメトリー実装
```

## ✅ 完了条件
- [ ] Python 3.11.9がインストールされている
- [ ] Cargo.tomlに依存関係が追加されている
- [ ] 開発ディレクトリ構造が準備されている
- [ ] 環境変数の設定方法を理解している

## 🚨 注意事項
- **Python 3.11固定必須** - AST安定性のため
- **pyo3::prepare_freethreaded_python()** を一度だけ呼ぶ
- GIL管理に注意（Phase 10.1cで詳細）

## ⏭️ 次のフェーズ
→ Phase 10.1c (パーサー統合実装)