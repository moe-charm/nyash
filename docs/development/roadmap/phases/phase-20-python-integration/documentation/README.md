# Documentation - ドキュメント

## 📋 概要

Python-Hakorune統合のドキュメント計画です。

## 📚 ドキュメント構成

### 1. ユーザーガイド

#### クイックスタート
```markdown
# Python統合クイックスタート

## インストール

hakorune-pythonプラグインをインストール:
```bash
cargo build --release --features python-integration
```

## 基本的な使い方

```hakorune
// Pythonコードを実行
local py = new PyRuntimeBox()
local result = py.eval("2 + 3")
console.log(result.to_string())  // "5"

// モジュールをインポート
local math = py.import("math")
local pi = math.getattr("pi")
console.log(pi.to_string())  // "3.14159..."
```
```

#### チュートリアル
- Python基本操作
- モジュールインポート
- オブジェクト操作
- エラーハンドリング
- パフォーマンス最適化

### 2. リファレンス

#### PyRuntimeBox API
```markdown
# PyRuntimeBox

Python実行環境を提供するBox。

## メソッド

### eval(code: StringBox) -> PyObjectBox
Pythonコードを評価し、結果を返す。

**引数:**
- `code`: 実行するPythonコード

**戻り値:**
- `PyObjectBox`: 実行結果

**例:**
```hakorune
local py = new PyRuntimeBox()
local result = py.eval("2 + 3")
```

### import(name: StringBox) -> PyObjectBox
Pythonモジュールをインポート。

**引数:**
- `name`: モジュール名

**戻り値:**
- `PyObjectBox`: インポートされたモジュール

**例:**
```hakorune
local math = py.import("math")
```
```

#### PyObjectBox API
```markdown
# PyObjectBox

Pythonオブジェクトを表すBox。

## メソッド

### getattr(name: StringBox) -> PyObjectBox
オブジェクトの属性を取得。

### setattr(name: StringBox, value: PyObjectBox)
オブジェクトの属性を設定。

### call(args: ArrayBox) -> PyObjectBox
オブジェクトを関数として呼び出し。

### to_string() -> StringBox
Pythonオブジェクトを文字列に変換。

### to_int() -> IntegerBox
Pythonオブジェクトを整数に変換。

### to_bool() -> BoolBox
Pythonオブジェクトを真偽値に変換。
```

### 3. 設計ドキュメント

#### アーキテクチャ
- 全体構成
- レイヤー構造
- データフロー

#### ABI仕様
- ハンドル管理
- 型変換ルール
- エラーハンドリング

#### GIL管理
- 獲得・解放ルール
- ネスト呼び出し
- マルチスレッド対応

### 4. 開発者ガイド

#### プラグイン開発
```markdown
# Python統合プラグイン開発ガイド

## プラグインの構造

```rust
// src/lib.rs
use hakorune_plugin_api::*;

#[no_mangle]
pub extern "C" fn nyplug_python_invoke(
    method_id: u32,
    args: *const TLV,
    args_len: usize,
    ret: *mut TLV,
) -> i32 {
    // 実装
}
```

## ビルド

```bash
cargo build --release --crate-type cdylib
```
```

#### コントリビューション
- コーディング規約
- テストガイドライン
- プルリクエストプロセス

### 5. トラブルシューティング

#### よくある問題

**Q: Pythonモジュールが見つからない**
```
A: PYTHONPATHを設定してください:
export PYTHONPATH=/path/to/modules:$PYTHONPATH
```

**Q: GILデッドロックが発生する**
```
A: ネスト呼び出しを避けるか、GILを明示的に解放してください。
```

**Q: メモリリークが発生する**
```
A: PyObjectBoxを適切にdropしているか確認してください。
```

## 📝 ドキュメント作成タスク

### Phase 1: 基本ドキュメント
- [ ] README.md（概要）
- [ ] QUICKSTART.md（クイックスタート）
- [ ] INSTALLATION.md（インストール）

### Phase 2: API リファレンス
- [ ] PyRuntimeBox API
- [ ] PyObjectBox API
- [ ] 型変換ルール

### Phase 3: チュートリアル
- [ ] 基本操作
- [ ] モジュールインポート
- [ ] オブジェクト操作
- [ ] エラーハンドリング

### Phase 4: 開発者向け
- [ ] アーキテクチャ設計
- [ ] プラグイン開発ガイド
- [ ] コントリビューションガイド

### Phase 5: その他
- [ ] トラブルシューティング
- [ ] パフォーマンスガイド
- [ ] ベストプラクティス

## 🎯 ドキュメント品質基準

### 必須要件
- [ ] コードサンプルが動作する
- [ ] 用語が一貫している
- [ ] エラーメッセージが明確
- [ ] プラットフォーム差異を記載

### 推奨要件
- [ ] 図表を含む
- [ ] ビデオチュートリアル
- [ ] 多言語対応（日本語・英語）
- [ ] 検索可能

## 📊 ドキュメント構成図

```
docs/
├── guides/
│   ├── python-integration/
│   │   ├── README.md           # 概要
│   │   ├── quickstart.md       # クイックスタート
│   │   ├── installation.md     # インストール
│   │   ├── tutorial/           # チュートリアル
│   │   │   ├── 01-basics.md
│   │   │   ├── 02-modules.md
│   │   │   ├── 03-objects.md
│   │   │   └── 04-errors.md
│   │   └── troubleshooting.md  # トラブルシューティング
│   └── ...
├── reference/
│   ├── python-integration/
│   │   ├── pyruntimebox-api.md # PyRuntimeBox API
│   │   ├── pyobjectbox-api.md  # PyObjectBox API
│   │   ├── type-conversion.md  # 型変換ルール
│   │   └── error-handling.md   # エラーハンドリング
│   └── ...
└── development/
    └── python-integration/
        ├── architecture.md      # アーキテクチャ
        ├── plugin-dev.md        # プラグイン開発
        └── contributing.md      # コントリビューション
```

## 🔗 関連リソース

### 公式ドキュメント
- [Python C API](https://docs.python.org/3/c-api/)
- [PyO3 Guide](https://pyo3.rs/)

### 参考実装
- Rust PyO3
- Node.js Python bindings
- Julia Python interop

## 🔗 関連ドキュメント

- [Phase 20 メインREADME](../README.md)
- [Planning](../planning/)
- [Core Implementation](../core-implementation/)
