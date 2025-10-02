# Testing - テスト計画

## 📋 概要

Python-Hakorune統合のテスト計画です。

## 🎯 テスト戦略

### テストレベル

1. **ユニットテスト**
   - 個別コンポーネントのテスト
   - 型変換のテスト
   - エラーハンドリングのテスト

2. **統合テスト**
   - Python-Hakorune連携テスト
   - FFI境界のテスト
   - GIL管理のテスト

3. **E2Eテスト**
   - 実用的なユースケーステスト
   - パフォーマンステスト
   - 互換性テスト

## 🧪 テストケース

### 1. 基本機能テスト

#### Python実行
```hakorune
// test_python_eval.hkr
local py = new PyRuntimeBox()
local result = py.eval("2 + 3")
assert(result.to_int() == 5)
```

#### モジュールインポート
```hakorune
// test_python_import.hkr
local py = new PyRuntimeBox()
local math = py.import("math")
local pi = math.getattr("pi")
assert(pi.to_string() == "3.14159...")
```

#### オブジェクト操作
```hakorune
// test_python_object.hkr
local py = new PyRuntimeBox()
local obj = py.eval("{'x': 10, 'y': 20}")
local x = obj.getattr("x")
assert(x.to_int() == 10)
```

### 2. 型変換テスト

```hakorune
// test_type_conversion.hkr
local py = new PyRuntimeBox()

// Bool変換
local py_true = py.eval("True")
assert(py_true.to_bool() == true)

// Int変換
local py_int = py.eval("42")
assert(py_int.to_int() == 42)

// String変換
local py_str = py.eval("'hello'")
assert(py_str.to_string() == "hello")

// List変換
local py_list = py.eval("[1, 2, 3]")
// ... ArrayBox への変換テスト
```

### 3. エラーハンドリングテスト

```hakorune
// test_python_error.hkr
local py = new PyRuntimeBox()

// Python例外のキャッチ
try {
    py.eval("1 / 0")  // ZeroDivisionError
} catch(e) {
    assert(e.contains("ZeroDivisionError"))
}
```

### 4. GIL管理テスト

```rust
// tests/gil_management.rs
#[test]
fn test_gil_acquisition() {
    // GILの正しい獲得・解放
}

#[test]
fn test_nested_gil_calls() {
    // ネストしたPython呼び出し
}

#[test]
fn test_concurrent_access() {
    // 複数スレッドからのアクセス
}
```

### 5. メモリ管理テスト

```rust
// tests/memory_management.rs
#[test]
fn test_no_memory_leak() {
    // メモリリークがないことを確認
    for _ in 0..1000 {
        let obj = create_python_object();
        drop(obj);
    }
    // メモリ使用量が増加していないことを確認
}

#[test]
fn test_reference_counting() {
    // Pythonオブジェクトの参照カウント
}
```

### 6. パフォーマンステスト

```rust
// benches/python_integration.rs
#[bench]
fn bench_python_call_overhead(b: &mut Bencher) {
    // Python呼び出しのオーバーヘッド測定
}

#[bench]
fn bench_type_conversion(b: &mut Bencher) {
    // 型変換のパフォーマンス測定
}
```

## 📊 テスト環境

### サポート対象

| 環境 | Python | Platform | 備考 |
|------|--------|----------|------|
| Linux | 3.8+ | x86_64 | 優先 |
| macOS | 3.8+ | x86_64, arm64 | 優先 |
| Windows | 3.8+ | x86_64 | 後段 |

### CI/CD統合

```yaml
# .github/workflows/python-integration.yml
name: Python Integration Tests

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        python-version: ['3.8', '3.9', '3.10', '3.11']

    steps:
      - uses: actions/checkout@v2
      - uses: actions/setup-python@v2
        with:
          python-version: ${{ matrix.python-version }}
      - name: Run tests
        run: cargo test --features python-integration
```

## 🔧 テストツール

### 1. ユニットテスト
- Cargo test framework
- `#[test]` アトリビュート
- アサーションマクロ

### 2. 統合テスト
- `tests/` ディレクトリ
- E2Eシナリオ

### 3. ベンチマーク
- Criterion.rs
- パフォーマンス回帰検出

### 4. メモリリークチェック
- Valgrind
- AddressSanitizer
- Python memory profiler

## 📋 テストチェックリスト

### 基本機能
- [ ] Python eval実行
- [ ] モジュールインポート
- [ ] オブジェクト属性アクセス
- [ ] メソッド呼び出し
- [ ] 例外ハンドリング

### 型変換
- [ ] bool変換
- [ ] int変換
- [ ] string変換
- [ ] list変換
- [ ] dict変換
- [ ] None処理

### FFI境界
- [ ] Handle管理
- [ ] TLV型変換
- [ ] エラー伝搬

### GIL管理
- [ ] GIL獲得・解放
- [ ] ネスト呼び出し
- [ ] マルチスレッド

### メモリ管理
- [ ] 参照カウント
- [ ] リーク検出
- [ ] 循環参照処理

### パフォーマンス
- [ ] 呼び出しオーバーヘッド
- [ ] 型変換コスト
- [ ] メモリ使用量

### プラットフォーム
- [ ] Linux (x86_64)
- [ ] macOS (x86_64)
- [ ] macOS (arm64)
- [ ] Windows (x86_64)

## ⚠️ 既知の課題

### 1. GILデッドロック
- **問題**: ネスト呼び出しでデッドロックの可能性
- **対策**: GIL獲得・解放のログ、タイムアウト検出

### 2. メモリリーク
- **問題**: 参照カウントの管理ミス
- **対策**: 定期的なリークチェック、自動テスト

### 3. プラットフォーム固有問題
- **問題**: Python配布形態の違い
- **対策**: CI環境での網羅的テスト

## 🔗 関連ドキュメント

- [Phase 20 メインREADME](../README.md)
- [Core Implementation](../core-implementation/)
- [Design Documents](../design/)
