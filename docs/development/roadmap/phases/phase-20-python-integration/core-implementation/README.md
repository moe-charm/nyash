# Core Implementation - コア実装

## 📋 概要

Python-Hakorune統合のコア実装計画です。

## 📁 ファイル一覧

- **[implementation-roadmap.md](implementation-roadmap.md)** - Python実装ロードマップ

## 🎯 実装コンポーネント

### 1. PyRuntimeBox（Python実行環境）

```hakorune
box PyRuntimeBox {
    // 初期化・終了
    init()
    shutdown()

    // コード実行
    eval(code: StringBox) -> PyObjectBox
    exec(code: StringBox)

    // モジュール
    import(name: StringBox) -> PyObjectBox
}
```

### 2. PyObjectBox（Pythonオブジェクト）

```hakorune
box PyObjectBox {
    // 属性アクセス
    getattr(name: StringBox) -> PyObjectBox
    setattr(name: StringBox, value: PyObjectBox)

    // 呼び出し
    call(args: ArrayBox) -> PyObjectBox

    // 変換
    to_string() -> StringBox
    to_int() -> IntegerBox
    to_bool() -> BoolBox
}
```

### 3. 相互運用レイヤー

#### Hakorune → Python
- BoxCall → CPython C API
- 型変換: Hakorune Box → PyObject*
- エラーハンドリング

#### Python → Hakorune
- CPython拡張モジュール（`hakorunert`）
- 型変換: PyObject* → Hakorune Box
- GIL管理

## 🏗️ アーキテクチャ

### レイヤー構造

```
┌─────────────────────────────┐
│   Hakorune Application      │
├─────────────────────────────┤
│   PyRuntimeBox/PyObjectBox  │
├─────────────────────────────┤
│   FFI/Plugin Interface      │
├─────────────────────────────┤
│   CPython C API             │
├─────────────────────────────┤
│   CPython Runtime           │
└─────────────────────────────┘
```

### プラグイン化

- `hakorune-python-plugin` (cdylib/staticlib)
- `nyplug_python_invoke` FFI関数
- 動的ロード/静的リンク対応

## 🔧 技術的詳細

### 1. メモリ管理

#### Python側
- `Py_INCREF` / `Py_DECREF`
- Boxライフサイクルとの連携
- 循環参照の回避

#### Hakorune側
- Handle管理（TLV tag=8）
- Arc<PyObject> でのラップ
- Drop時の自動DECREF

### 2. GIL管理

```rust
// 基本パターン
{
    let gil = Python::acquire_gil();
    let py = gil.python();

    // Python操作
    let result = py.eval(code, None, None)?;

    // GILは自動解放
}
```

### 3. エラーハンドリング

- Python例外 → Hakorune文字列エラー（tag=6）
- トレースバック情報の保持
- 双方向のエラー伝搬

## 📊 実装ステータス

| コンポーネント | ステータス | 備考 |
|--------------|----------|------|
| PyRuntimeBox設計 | ✅ 完了 | - |
| PyObjectBox設計 | ✅ 完了 | - |
| FFI統合 | 📅 未実装 | - |
| GIL管理 | 📅 未実装 | - |
| エラーハンドリング | 📅 未実装 | - |
| メモリ管理 | 📅 未実装 | - |

## ⚠️ リスク要因

### 1. GILデッドロック
- 入口/出口での厳格な管理
- ネスト呼び出し時の方針
- デッドロック検出機能

### 2. 参照カウントリーク
- BoxライフサイクルでのDECREF保証
- リークテストの追加
- デバッグツール整備

### 3. パフォーマンス
- FFIオーバーヘッド
- GIL待機時間
- 最適化戦略

## 🔗 関連ドキュメント

- [Phase 20 メインREADME](../README.md)
- [Planning](../planning/)
- [Design Documents](../design/)
