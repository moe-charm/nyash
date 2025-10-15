# [Archived] 旧 Phase 10.1 - Python統合計画（ChatGPT5高速開発版）
最終更新: 2025-08-27 ／ 状態: Archived（Phase 10.5 に統合）

現行の計画は Phase 10.5 のドキュメントに集約しています。最新は以下を参照してください:
- Phase 10.5 Index: ./INDEX.md
- 10.5a – Python ABI 設計: ./10.5a-ABI-DESIGN.md
- 10.5b – ネイティブビルド基盤: ./10.5b-native-build-consolidation.md

## 🚀 概要：2週間での爆速実装（当時案）

ChatGPT5の最小Box設計により、元の1ヶ月計画を**2週間**に圧縮。Nyash既存アーキテクチャ（MirBuilder 100%実装済み、HandleRegistry 80%実装済み）を最大活用。

## 📦 ChatGPT5の6つの必須Box（最小実装）

### 1. **PythonParserBox** - CPython AST取得（3日）
```rust
// 既存: pyo3統合済み
// 追加: JSON出力とバージョン固定
pub struct PythonParserBox {
    base: BoxBase,
    py_helper: Arc<Mutex<PyHelper>>,
    version: String, // "3.11"固定
}

// メソッド（最小限）
- parse_to_json(src: String) -> String  // ast.parse() → JSON
- get_version() -> String               // "3.11"
```

### 2. **Py2NyASTBox** - AST変換（3日）
```rust
// 新規実装
pub struct Py2NyASTBox {
    base: BoxBase,
    normalizer: AstNormalizer,
}

// メソッド（制御フロー正規化）
- convert(json: String) -> NyashAst
- normalize_for_else(ast: &mut PyAst)  // for/else → if分岐
- normalize_comprehensions(ast: &mut PyAst)
```

### 3. **MirBuilderBox** - MIR生成（0日 - 既存活用）
```rust
// 既存実装100%活用
// 追加: Python由来フラグのみ
pub struct MirBuilderBox {
    // 既存フィールド
    is_python_origin: bool,  // 追加
}
```

### 4. **BoundaryBox** - 型変換（2日）
```rust
// Python版のHandleRegistry相当
pub struct BoundaryBox {
    base: BoxBase,
    handle_registry: Arc<Mutex<HandleRegistry>>, // 既存80%活用
}

// メソッド
- py_to_jit(py_val: PyValBox) -> JitValue
- jit_to_py(jit_val: JitValue) -> PyValBox
- register_handle(obj: Arc<dyn NyashBox>) -> u64
```

### 5. **PyRuntimeBox** - 実行制御（2日）
```rust
pub struct PyRuntimeBox {
    base: BoxBase,
    fallback_stats: FallbackStats,
}

// メソッド（関数単位フォールバック）
- execute_function(name: &str, args: Vec<JitValue>) -> JitValue
- should_fallback(func_ast: &PyAst) -> bool  // Phase1機能判定
- fallback_to_cpython(code: &str) -> PyObject
```

### 6. **ObservabilityBox** - 統計収集（1日）
```rust
// 既存のJIT統計システム（70%実装済み）を拡張
pub struct ObservabilityBox {
    base: BoxBase,
    stats_collector: StatsCollector,
}

// JSONLフォーマット出力
- log_attempt(module: &str, func: &str, compiled: bool, reason: Option<&str>)
- output_jsonl() -> String
```

## 🗓️ 実装タイムライン（2週間）

### Week 1: 基盤実装（7日）
- **Day 1-3**: PythonParserBox実装
  - pyo3統合（既存活用）
  - Python 3.11固定
  - JSON出力実装
  
- **Day 4-6**: Py2NyASTBox実装  
  - 制御フロー正規化
  - for/else, while/else変換
  - Phase1機能のみサポート

- **Day 7**: ObservabilityBox実装
  - 既存JIT統計拡張
  - JSONLフォーマット

### Week 2: 統合と検証（7日）
- **Day 8-9**: BoundaryBox実装
  - HandleRegistry活用
  - 型変換ルール確立

- **Day 10-11**: PyRuntimeBox実装
  - 関数単位フォールバック
  - CPython連携

- **Day 12-13**: 統合テスト
  - Differential Testing
  - ベンチマーク実行

- **Day 14**: ドキュメント・リリース
  - 使用例作成
  - パフォーマンス測定

## 📊 既存アーキテクチャとの整合性

### 活用率
- **MirBuilderBox**: 100%（変更なし）
- **HandleRegistry**: 80%（BoundaryBoxで再利用）
- **JIT統計**: 70%（ObservabilityBoxで拡張）
- **VM/JIT実行**: 100%（そのまま使用）

### 新規実装
- **PythonParserBox**: 30%（pyo3部分は既存）
- **Py2NyASTBox**: 100%新規
- **PyRuntimeBox**: 100%新規

## 🎯 Phase 1でサポートする機能（Codex先生推奨）

### 必須実装
1. **LEGB + locals/freevars** - スコーピング規則
2. **デフォルト引数の評価タイミング** - 定義時評価
3. **イテレータベースのfor文**
4. **for/else + while/else**
5. **Python真偽値判定**
6. **短絡評価**

### サポートする文
- def（関数定義）
- if/elif/else
- for（else節対応）
- while（else節対応）
- break/continue
- return

### サポートする式
- 算術演算子（+,-,*,/,//,%）
- 比較演算子（==,!=,<,>,<=,>=）
- 論理演算子（and,or,not）
- 関数呼び出し
- リテラル（数値/文字列/bool）

## 📈 成功指標（2週間後）

### 定量的
- **関数コンパイル率**: 70%以上（Phase 1機能）
- **実行速度**: 純Pythonループで2倍以上
- **メモリ効率**: CPython比50%削減

### 定性的  
- **統計可視化**: JSONL形式で全実行を記録
- **デバッグ容易性**: 関数単位でフォールバック理由明示
- **将来拡張性**: Phase 2-4への明確な道筋

## 🔧 実装例（最終形）

```nyash
// Nyashから使用
local py = new PythonParserBox()
local converter = new Py2NyASTBox()
local builder = new MirBuilderBox()
local runtime = new PyRuntimeBox()
local stats = new ObservabilityBox()

// Pythonコードをコンパイル・実行
local code = "def fib(n): return n if n <= 1 else fib(n-1) + fib(n-2)"
local json_ast = py.parse_to_json(code)
local ny_ast = converter.convert(json_ast)
local mir = builder.build(ny_ast)

// 実行（自動的にJIT/VMで高速化）
local result = runtime.execute_function("fib", [10])
print(result)  // 55

// 統計出力
print(stats.output_jsonl())
// {"mod":"test","func":"fib","attempt":1,"jitted":true,"native":true}
```

## 🚨 重要な設計判断

### 1. 関数単位の境界
- ファイル単位ではなく**関数単位**でコンパイル/フォールバック
- 未対応機能を含む関数のみCPython実行

### 2. Python 3.11固定
- AST安定性の確保
- 将来のバージョンアップは別Phase

### 3. 箱境界の明確化  
- 各Boxは単一責任
- 相互依存を最小化
- テスト可能な粒度

### 4. 既存資産の最大活用
- MirBuilder/VM/JITはそのまま使用
- 新規実装は変換層のみ

## 🎉 期待される成果

### 即時的効果（2週間後）
- Pythonコードの70%がNyashで高速実行
- バグ検出力の飛躍的向上（Differential Testing）
- 統計による最適化ポイントの可視化

### 長期的効果
- Python→Nyash→Native の世界初パイプライン確立
- Nyash言語の成熟度向上
- エコシステムの爆発的拡大

## 📝 次のステップ

1. **Phase 10.7完了確認** - JIT統計JSONの安定化
2. **PythonParserBox実装開始** - pyo3統合から着手
3. **テストコーパス準備** - Python標準ライブラリから抜粋

---

**作成者**: Claude（Claude Code）  
**承認者**: ChatGPT5（予定）  
**開始予定**: Phase 10.7完了直後
