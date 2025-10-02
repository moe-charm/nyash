# リスクと対策

**作成**: 2025-10-02
**ソース**: ChatGPT Pro UltraThink Mode
**用途**: Python統合における既知のリスクと対策

---

## 📋 概要

Python-Hakorune統合における技術的リスク、運用リスク、およびその対策をまとめます。

---

## 🔥 高リスク項目

### 1. GILデッドロック

#### リスク詳細
- **問題**: Hakorune並列実行とPython GILのズレ
- **発生条件**: Hakorune側マルチスレッド + Python呼び出しのネスト
- **影響**: プログラム全体がハング、デバッグ困難

#### 対策

**アーキテクチャレベル**:
```rust
// PyRuntimeBoxを専用スレッドで実行
pub struct PyRuntimeBox {
    thread: JoinHandle<()>,
    tx: Sender<PyCommand>,
    rx: Receiver<PyResult>,
}

impl PyRuntimeBox {
    pub fn init() -> Self {
        let (tx, rx_cmd) = channel();
        let (tx_result, rx) = channel();

        let thread = thread::spawn(move || {
            // このスレッドでGILを保持
            let gil = Python::acquire_gil();
            let py = gil.python();

            loop {
                match rx_cmd.recv() {
                    Ok(PyCommand::Import(name)) => {
                        let result = py.import(&name);
                        tx_result.send(PyResult::Module(result)).unwrap();
                    }
                    Ok(PyCommand::Shutdown) => break,
                    Err(_) => break,
                }
            }
        });

        PyRuntimeBox { thread, tx, rx }
    }
}
```

**ガードレール**:
```toml
[debug.gil]
# GIL獲得・解放のログ
trace = true

# タイムアウト検出
timeout = 5.0  # 5秒

# デッドロック検出
detect_deadlock = true
```

**モニタリング**:
```bash
# GILトレース有効化
export HAKO_TRACE_GIL=1

# タイムアウト設定
export HAKO_GIL_TIMEOUT=5.0
```

#### 回避策
1. **専用スレッド**でPython実行を隔離
2. **メッセージパッシング**でHakorune-Python間通信
3. **タイムアウト**で異常検出
4. **ログ**で原因追跡

---

### 2. メモリリーク

#### リスク詳細
- **問題**: Python参照カウント管理ミス
- **発生条件**: `Py_INCREF`/`Py_DECREF`の不一致
- **影響**: メモリ使用量増大、最悪OOM

#### 対策

**Arc管理**:
```rust
// PythonオブジェクトをArcでラップ
pub struct PyObjectBox {
    inner: Arc<PyObjectInner>,
}

struct PyObjectInner {
    py_obj: Py<PyAny>,
}

impl Drop for PyObjectInner {
    fn drop(&mut self) {
        // 確実にDECREF
        Python::with_gil(|py| {
            self.py_obj.as_ref(py);
        });
    }
}
```

**weakref.finalize**:
```python
# Python側で自動クリーンアップ
import weakref

def create_hakorune_object(obj):
    # finalizer登録
    finalizer = weakref.finalize(obj, cleanup_callback, obj_id)
    return obj
```

**リーク検出**:
```toml
[debug.memory]
# メモリトラッキング
track = true

# リーク検出
detect_leaks = true

# 定期チェック
check_interval = 10.0  # 10秒
```

**ツール**:
```bash
# Valgrind
valgrind --leak-check=full ./hakorune script.hkr

# AddressSanitizer
RUSTFLAGS="-Z sanitizer=address" cargo build

# Python memory profiler
export PYTHONTRACEMALLOC=1
```

#### 回避策
1. **Arc/Drop**で自動管理
2. **weakref.finalize**で循環参照対策
3. **定期チェック**で早期発見
4. **テスト**で継続監視

---

### 3. C拡張モジュールの扱い

#### リスク詳細
- **問題**: numpy等のC拡張は特殊な扱いが必要
- **発生条件**: ネイティブコードの直接呼び出し
- **影響**: セグフォ、未定義動作

#### 対策

**Capability制御**:
```toml
[box.PyRuntimeBox.capabilities.native]
# C拡張の許可
allow_native = true

# ホワイトリスト
allow_modules = [
    "numpy",
    "pandas",
    "_ctypes"
]

# ブラックリスト
deny_modules = [
    "ctypes",  # 任意ネイティブコード実行
    "cffi"
]
```

**サンドボックス**（将来）:
```toml
[sandbox]
# ネイティブコードをサンドボックスで実行
enable = true
mode = "seccomp"  # Linux
```

**検証**:
```rust
// ロード前に検証
fn verify_module_safety(module_name: &str) -> Result<()> {
    // シグネチャ確認
    // 既知の安全なモジュールか
    // ブラックリスト確認
}
```

#### 回避策
1. **ホワイトリスト**で既知の安全なモジュールのみ許可
2. **サンドボックス**で隔離実行（将来）
3. **検証**でロード前チェック

---

## ⚠️ 中リスク項目

### 4. パフォーマンスオーバーヘッド

#### リスク詳細
- **問題**: FFI境界・GIL・型変換のコスト
- **影響**: 性能劣化

#### 対策

**最適化**:
```rust
// 型変換キャッシュ
struct TypeCache {
    string_to_pyobject: HashMap<String, Py<PyString>>,
    int_to_pyobject: HashMap<i64, Py<PyLong>>,
}

impl TypeCache {
    fn get_or_create_string(&mut self, py: Python, s: &str) -> Py<PyString> {
        self.string_to_pyobject
            .entry(s.to_string())
            .or_insert_with(|| PyString::new(py, s).into())
            .clone()
    }
}
```

**バッチ処理**:
```rust
// 複数呼び出しをまとめる
fn batch_call(funcs: Vec<PyFunctionBox>, args: Vec<Vec<Value>>) -> Vec<Value> {
    Python::with_gil(|py| {
        funcs.iter().zip(args.iter())
            .map(|(f, a)| f.exec_with_gil(py, a))
            .collect()
    })
}
```

**ベンチマーク**:
```bash
# パフォーマンステスト
cargo bench --features python-integration
```

---

### 5. プラットフォーム固有問題

#### リスク詳細
- **問題**: Python配布形態の違い（Linux/macOS/Windows）
- **影響**: ビルド・実行時エラー

#### 対策

**プラットフォーム別設定**:
```toml
# Linux
[target.x86_64-unknown-linux-gnu]
python-lib = "python3.11"
python-path = "/usr/lib/python3.11"

# macOS
[target.x86_64-apple-darwin]
python-lib = "python3.11"
python-path = "/usr/local/opt/python@3.11"

# Windows
[target.x86_64-pc-windows-msvc]
python-lib = "python311"
python-path = "C:/Python311"
```

**CI/CD**:
```yaml
# .github/workflows/python-integration.yml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    python-version: ['3.8', '3.9', '3.10', '3.11']
```

---

### 6. 例外伝播の複雑さ

#### リスク詳細
- **問題**: Python例外とHakoruneエラーの境界
- **影響**: エラーハンドリングの困難さ

#### 対策

**統一ErrorBox**:
```hakorune
box ErrorBox {
    type: StringBox         // "ZeroDivisionError"
    message: StringBox      // "division by zero"
    traceback: StringBox    // フルスタックトレース
    source: StringBox       // "python" | "hakorune"
}
```

**変換層**:
```rust
fn convert_py_exception(py_err: &PyErr) -> ErrorBox {
    ErrorBox {
        type_: py_err.get_type().name().to_string(),
        message: py_err.to_string(),
        traceback: format_traceback(py_err),
        source: "python".to_string(),
    }
}
```

---

## 📊 低リスク項目

### 7. ドキュメント不足

#### 対策
- 段階的にドキュメント整備
- コードサンプル充実
- チュートリアル作成

### 8. テストカバレッジ

#### 対策
- ユニットテスト追加
- 統合テスト充実
- E2Eテスト自動化

---

## 🛡️ リスク管理マトリックス

| リスク | 深刻度 | 発生確率 | 優先度 | 対策状況 |
|-------|-------|---------|-------|---------|
| GILデッドロック | 高 | 中 | 最優先 | 設計段階で対策 |
| メモリリーク | 高 | 中 | 最優先 | Arc/Drop自動化 |
| C拡張問題 | 高 | 低 | 高 | ホワイトリスト |
| パフォーマンス | 中 | 高 | 中 | 最適化計画 |
| プラットフォーム | 中 | 中 | 中 | CI/CD網羅 |
| 例外伝播 | 中 | 低 | 低 | ErrorBox統一 |
| ドキュメント | 低 | 高 | 低 | 段階的整備 |
| テストカバレッジ | 低 | 中 | 低 | 継続改善 |

---

## 🔍 監視・検出

### 自動検出

```toml
[monitoring]
# GIL監視
gil.timeout = 5.0
gil.alert_on_timeout = true

# メモリ監視
memory.threshold = 100_000_000  # 100MB
memory.alert_on_threshold = true

# パフォーマンス監視
performance.slow_call_threshold = 1.0  # 1秒
performance.alert_on_slow = true
```

### ログ

```bash
# すべての監視ログ
export HAKO_TRACE_GIL=1
export HAKO_TRACE_MEMORY=1
export HAKO_TRACE_PERFORMANCE=1
```

---

## 📝 インシデント対応

### 1. GILデッドロック発生時

```bash
# 1. ログ確認
cat /tmp/hakorune-debug.log | grep GIL

# 2. スタックトレース取得
kill -QUIT <pid>

# 3. デバッガー接続
gdb -p <pid>
```

### 2. メモリリーク発生時

```bash
# 1. メモリ使用量確認
ps aux | grep hakorune

# 2. Valgrind実行
valgrind --leak-check=full ./hakorune script.hkr

# 3. Python側確認
export PYTHONTRACEMALLOC=1
```

---

## 🔗 関連ドキュメント

- [強化版アーキテクチャv2](enhanced-architecture-v2.md) - 設計詳細
- [マイルストーン](../planning/milestones.md) - 実装計画
- [メタ設定例](meta-config-examples.md) - 設定例
- [Phase 20 README](../README.md) - 全体概要

---

**最終更新**: 2025-10-02
**作成者**: ChatGPT Pro (UltraThink Mode)
**ステータス**: リスク管理ドキュメント
