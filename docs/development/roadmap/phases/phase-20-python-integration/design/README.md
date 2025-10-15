# Design - 設計ドキュメント

## 📋 概要

Python-Hakorune統合の設計ドキュメント集です。

## 📁 ファイル一覧

### 🌟 最新設計（2025-10-02追加）
- **[enhanced-architecture-v2.md](enhanced-architecture-v2.md)** ⭐必読 - ChatGPT Pro UltraThink強化版アーキテクチャ
- **[meta-config-examples.md](meta-config-examples.md)** - hako.toml設定リファレンス
- **[risks-and-mitigations.md](risks-and-mitigations.md)** - リスク管理ドキュメント

### ABI・FFI設計
- **[abi-design.md](abi-design.md)** - Python-Hakorune ABI設計
- **[handle-first-plugininvoke-plan.md](handle-first-plugininvoke-plan.md)** - Handle-First PluginInvoke設計

### ビルド・実行
- **[native-build-consolidation.md](native-build-consolidation.md)** - ネイティブビルド基盤設計

## 🎯 設計の核心

### 1. ABI設計方針

#### ハンドル管理
- **TLV tag=8**: Python object handle
- **type_id + instance_id**: 箱の識別
- **Arc<PyObject>**: Rust側での管理

#### 型変換

| Hakorune | Python | 備考 |
|---------|--------|------|
| BoolBox | bool | 真偽値 |
| IntegerBox | int | 整数 |
| StringBox | str | 文字列 |
| ArrayBox | list | 配列 |
| MapBox | dict | 辞書 |
| PyObjectBox | object | 任意のPythonオブジェクト |

### 2. Handle-First 設計

#### 原則
- 第一引数（a0）は常にハンドル
- `nyash.handle.of(receiver)` で変換
- TLV統一: String/Integer以外はHandle（tag=8）

#### メリット
- 型安全性の向上
- 統一されたインターフェース
- デバッグの容易さ

### 3. GIL管理戦略

#### 基本方針
- **birth/invoke/decRef中はGIL確保**
- **AOTでも同等の処理**
- **ネスト呼び出しの安全性保証**

#### 実装パターン
```rust
pub fn invoke_python_method(handle: Handle, method: &str, args: &[Value]) -> Result<Value> {
    // GIL獲得
    let gil = Python::acquire_gil();
    let py = gil.python();

    // Pythonオブジェクト取得
    let obj = get_pyobject_from_handle(handle)?;

    // メソッド呼び出し
    let result = obj.call_method(py, method, args, None)?;

    // 結果を変換
    let value = pyobject_to_value(result)?;

    Ok(value)
    // GILは自動解放
}
```

## 🏗️ アーキテクチャ概要

### レイヤー構造

```
┌─────────────────────────────────┐
│   Hakorune Application          │
├─────────────────────────────────┤
│   Python Integration Layer      │
│   - PyRuntimeBox                │
│   - PyObjectBox                 │
├─────────────────────────────────┤
│   FFI/Plugin Interface          │
│   - Handle-First design         │
│   - TLV type system            │
├─────────────────────────────────┤
│   CPython C API                 │
├─────────────────────────────────┤
│   CPython Runtime               │
└─────────────────────────────────┘
```

### プラグイン構成

```
hakorune-python-plugin/
├── src/
│   ├── lib.rs              # FFI entry point
│   ├── runtime.rs          # PyRuntimeBox
│   ├── object.rs           # PyObjectBox
│   ├── conversion.rs       # 型変換
│   └── gil.rs              # GIL管理
├── Cargo.toml
└── build.rs
```

## 🔧 ネイティブビルド戦略

### AOT/EXEパイプライン

```
Hakorune Source (.hkr)
    ↓ Parse
AST
    ↓ Lower
MIR
    ↓ JIT Compile
CLIF IR
    ↓ Object Generation
Object File (.o)
    ↓ Link (with libhakorunert.a + Python libs)
Native Executable
```

### クロスプラットフォーム対応

| Platform | Python Library | 備考 |
|----------|---------------|------|
| Linux | libpython3.X.so | 動的リンク |
| macOS | libpython3.X.dylib | 動的リンク |
| Windows | python3X.dll | 動的リンク |

## 📊 設計決定事項

### 1. Embedding vs Extending

**決定**: Embedding優先

理由：
- HakoruneからPythonを制御
- デプロイが簡単
- ユーザー体験が良い

### 2. 静的 vs 動的リンク

**決定**: 動的リンク優先、静的リンクはオプション

理由：
- Python標準の配布方法
- ライセンス問題の回避
- 柔軟性の確保

### 3. サポート対象Python

**決定**: Python 3.8以降

理由：
- 型ヒントの充実
- 十分な普及
- メンテナンス負荷

## ⚠️ リスク要因

### 1. CPython依存
- バージョン互換性
- プラットフォーム差異
- ビルド環境の複雑さ

### 2. パフォーマンス
- FFIオーバーヘッド
- GIL待機時間
- メモリコピーコスト

### 3. デバッグ困難性
- 言語境界を越えるエラー
- スタックトレースの複雑さ
- メモリリークの追跡

## 🔗 関連ドキュメント

- [Phase 20 メインREADME](../README.md)
- [Planning](../planning/)
- [Core Implementation](../core-implementation/)
