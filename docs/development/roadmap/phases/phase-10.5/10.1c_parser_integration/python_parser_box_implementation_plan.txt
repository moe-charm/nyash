# PythonParserBox実装計画（統合版）
更新日: 2025-08-27

## 🎯 エキスパートからの統合フィードバック

### 最重要ポイント（両エキスパートが一致）
1. **関数単位のフォールバック戦略**
   - ファイル全体でなく関数レベルでコンパイル/フォールバックを切り替え
   - 未対応機能を含む関数はCPython exec、対応済み関数はNyash MIR/JIT

2. **Python 3.11固定**
   - AST安定性の確保（3.8 Constant統一、3.10 match/case、3.12位置情報）
   - `py_version`と`ast_format`をJSON IRに埋め込む

3. **意味論の正確な実装が最優先**
   - 最適化より先にPython互換性を確保
   - 特に: イテレータプロトコル、真偽値判定、スコーピング規則（LEGB）

4. **GIL管理の最小化**
   - Python側でJSON生成（`ast.NodeVisitor` + `json.dumps`）
   - Rust側で解析（GIL外で実行）
   - `py.allow_threads(|| { ... })`で重い処理をGIL外実行

5. **テレメトリー重視**
   - 未対応ノードの記録（`support_level`フィールド）
   - フォールバック率の計測
   - ソース位置情報の保持（`lineno/col_offset/end_*`）

### Differential Testing戦略
- **世界中のPythonコードがNyashのテストケース**
- CPythonを「オラクル」として使用
- 出力、戻り値、例外を比較
- Grammar-based fuzzing（Hypothesis活用）

## 技術的実装方針

### 1. CPythonパーサー統合（pyo3使用）
```rust
// Cargo.toml
[dependencies]
pyo3 = { version = "0.22", features = ["auto-initialize"] }
pyo3-numpy = "0.22"  // NumPy連携用

// 初期化（一度だけ）
pyo3::prepare_freethreaded_python();

// Python側ヘルパー（embedded）
const PYTHON_HELPER: &str = r#"
import ast
import json
import sys

def parse_to_json(code, filename="<string>", mode="exec"):
    tree = ast.parse(code, filename, mode)
    return json.dumps(ast_to_dict(tree))

def ast_to_dict(node):
    result = {}
    
    # 必須フィールド
    result['node_type'] = node.__class__.__name__
    result['py_version'] = f"{sys.version_info.major}.{sys.version_info.minor}"
    
    # 位置情報（エラー診断用）
    if hasattr(node, 'lineno'):
        result['lineno'] = node.lineno
        result['col_offset'] = node.col_offset
        if hasattr(node, 'end_lineno'):  # Python 3.8+
            result['end_lineno'] = node.end_lineno
            result['end_col_offset'] = node.end_col_offset
    
    # サポートレベル（Nyash側で設定）
    result['support_level'] = 'unknown'
    
    # ASTフィールド
    if isinstance(node, ast.AST):
        for field in node._fields:
            value = getattr(node, field)
            result[field] = ast_to_dict(value)
    elif isinstance(node, list):
        return [ast_to_dict(x) for x in node]
    else:
        return node
        
    return result
"#;
```

### 2. 最小実装セット（Phase 1: Must-Have）
```
Phase 1 意味論の必須要素（Codex先生強調）:
- LEGB + locals/freevars（スコーピング）
- デフォルト引数の評価タイミング（定義時）
- イテレータベースのfor文
- for/else + while/else（Python独特）
- Python真偽値判定（__bool__ → __len__）
- 短絡評価（and/or）

Phase 1 AST構造:
├─ Module (py_version, ast_format)
├─ FunctionDef (name, args, body, decorator_list=[])
│   └─ arguments (args, defaults, kwonlyargs=[], kw_defaults=[])
├─ Return (value)
├─ Assign (targets, value) 
├─ AugAssign (target, op, value)  # +=, -=等
└─ Expr (value)

Phase 1 式:
├─ BinOp (left, op, right)
│   └─ ops: Add, Sub, Mult, Div, FloorDiv, Mod, Pow
├─ Compare (left, ops, comparators)
│   └─ ops: Eq, NotEq, Lt, LtE, Gt, GtE, Is, IsNot
├─ BoolOp (op, values)  # and/or
├─ UnaryOp (op, operand)  # not, -, +
├─ Call (func, args, keywords=[])
├─ Name (id, ctx=Load/Store/Del)
├─ Constant (value)  # Python 3.8+統一
└─ IfExp (test, body, orelse)  # 三項演算子

Phase 1 制御フロー:
├─ If (test, body, orelse)
├─ While (test, body, orelse)  # else節対応必須
├─ For (target, iter, body, orelse)  # else節対応必須
├─ Break
└─ Continue
```

### 3. 関数単位フォールバック戦略
```rust
// 関数単位のコンパイル判定
pub struct FunctionCompiler {
    supported_nodes: HashSet<&'static str>,
    telemetry: CompilationTelemetry,
}

impl FunctionCompiler {
    pub fn can_compile(&self, func_def: &PythonAst) -> CompileResult {
        let mut visitor = SupportChecker::new(&self.supported_nodes);
        visitor.visit(func_def);
        
        if visitor.has_unsupported() {
            // CPython execへフォールバック
            CompileResult::Fallback {
                reason: visitor.unsupported_nodes(),
                location: func_def.location(),
            }
        } else {
            // Nyash MIR/JITへコンパイル
            CompileResult::Compile
        }
    }
    
    pub fn compile_module(&mut self, module: &PythonAst) -> ModuleUnit {
        let mut units = vec![];
        
        // モジュールトップレベルはPythonで実行（globals設定）
        units.push(ExecutionUnit::PythonExec(module.top_level));
        
        // 各関数を判定
        for func in module.functions() {
            match self.can_compile(func) {
                CompileResult::Compile => {
                    let mir = self.compile_to_mir(func);
                    units.push(ExecutionUnit::NyashFunction(mir));
                    self.telemetry.record_compiled(func.name);
                }
                CompileResult::Fallback { reason, location } => {
                    units.push(ExecutionUnit::PythonThunk(func));
                    self.telemetry.record_fallback(func.name, reason, location);
                }
            }
        }
        
        ModuleUnit { units }
    }
}
```

### 4. データ共有戦略
```rust
// NdArrayBox定義
pub struct NdArrayBox {
    base: BoxBase,
    py_array: Py<PyArray<f64, Dim<[usize; 2]>>>,  // Python側の参照保持
    // 操作時のみGIL取得してArrayViewを取る
}

impl NdArrayBox {
    pub fn to_view(&self) -> PyResult<ArrayView2<f64>> {
        Python::with_gil(|py| {
            let array = self.py_array.as_ref(py);
            Ok(array.readonly())
        })
    }
}
```

### 4. 実装ロードマップ

#### Phase 1: パーサー統合（1-2週間）
- [ ] pyo3セットアップとPythonParserBox骨格
- [ ] Python側parse_to_jsonヘルパー実装
- [ ] JSON→Nyash AST最小変換
- [ ] run()メソッド（CPython exec委譲）
- [ ] 例外変換（PyErr → NyashError）

#### Phase 2: MIR変換（2-4週間）
- [ ] AST→MIR変換器（最小セット）
- [ ] 数値演算プリミティブ実装
- [ ] スコープ解決（関数ローカル/グローバル）
- [ ] 基本的な制御フロー（If/While）

#### Phase 3: NumPy統合（並行可能）
- [ ] pyo3-numpy統合
- [ ] NdArrayBox実装
- [ ] ゼロコピーベンチマーク
- [ ] バッファプロトコル対応

#### Phase 4: 最適化と拡張
- [ ] 型特化とガード最適化
- [ ] 例外処理（try/except）
- [ ] クラス/メソッド対応
- [ ] import統合

## 性能目標（現実的な見積もり）

| コードタイプ | 期待される高速化 | 備考 |
|------------|----------------|------|
| 純Pythonループ | 2-10倍 | 型安定なホットパス |
| 関数呼び出し多 | 1.5-3倍 | インライン化効果 |
| NumPy処理中心 | 1.0-1.2倍 | 既に最適化済み |
| 動的特性多用 | 1.2-3倍 | ガード頻発で限定的 |

## 実装上の注意点（エキスパート推奨）

### 意味論の重要な違い（Phase 1で対応必須）

1. **制御フロー**
   - `for`文: イテレータプロトコル必須（`__iter__`/`__next__`）
   - `for/else`, `while/else`: breakしなかった場合のelse実行
   - 短絡評価: `and`は左がFalseなら右を評価しない

2. **スコープ規則（LEGB）**
   ```python
   # Local → Enclosing → Global → Builtins
   global_var = 1
   
   def outer():
       enclosing_var = 2
       
       def inner():
           local_var = 3
           nonlocal enclosing_var  # 明示的な宣言
           global global_var       # 明示的な宣言
   ```

3. **数値演算の違い**
   - `/`: Python 3では常にfloat（true division）
   - `//`: floor division（整数除算）
   - 大整数: デフォルトで無限精度
   - `is` vs `==`: オブジェクト同一性 vs 値の等価性

4. **関数定義の罠**
   ```python
   def f(x, y=[]): # デフォルト引数は定義時に1度だけ評価！
       y.append(x) # 全呼び出しで同じリストを共有
       return y
   ```

### GIL管理のベストプラクティス

```rust
// ❌ 悪い例: GILを長時間保持
let result = Python::with_gil(|py| {
    let ast = parse_python(py, code)?;
    let json = convert_to_json(py, ast)?;  // ここまでGIL必要
    let nyash_ast = parse_json(&json)?;    // GIL不要なのに保持
    compile_to_mir(nyash_ast)?             // GIL不要なのに保持
});

// ✅ 良い例: GILを最小限に
let json = Python::with_gil(|py| {
    let ast = parse_python(py, code)?;
    convert_to_json(py, ast)  // JSON生成まで
})?;

// GIL外で重い処理
let nyash_ast = parse_json(&json)?;
let mir = compile_to_mir(nyash_ast)?;

// 必要時のみ再取得
Python::with_gil(|py| {
    py.allow_threads(|| {
        // 時間のかかるRust処理
        optimize_mir(mir)
    })
})
```

### テレメトリーとデバッグ

```rust
// 環境変数で制御
NYASH_PYTHONPARSER_TELEMETRY=1      # 基本統計
NYASH_PYTHONPARSER_TELEMETRY=2      # 詳細ログ
NYASH_PYTHONPARSER_STRICT=1         # フォールバック時にパニック（CI用）

// 出力例
[PythonParser] Module: example.py
  Functions: 10 total
  Compiled: 7 (70%)
  Fallback: 3 (30%)
    - async_function: unsupported node 'AsyncFunctionDef' at line 23
    - generator_func: unsupported node 'Yield' at line 45  
    - class_method: unsupported node 'ClassDef' at line 67
```

## 次のステップ

### 即座に開始すべきこと

1. **Python 3.11環境固定**
   ```bash
   pyenv install 3.11.9
   pyenv local 3.11.9
   ```

2. **最小動作確認**
   ```python
   # test_minimal.py
   def add(x, y):
       return x + y
   
   result = add(10, 5)
   print(f"Result: {result}")  # → Nyashで15が出力されれば成功！
   ```

3. **テレメトリー基盤構築**
   - 未対応ノードの記録システム
   - フォールバック率の可視化
   - ソース位置情報の保持

4. **Differential Testingの準備**
   - CPythonとの出力比較フレームワーク
   - 標準出力、戻り値、例外のキャプチャ
   - テストコーパスの選定

### 成功の測定基準

| フェーズ | 目標 | 測定指標 |
|---------|------|----------|
| Phase 1 | 基本動作 | 簡単な数値計算の70%がコンパイル可能 |
| Phase 2 | 実用性 | scikit-learnの基本アルゴリズムが動作 |
| Phase 3 | 性能 | 純Pythonループで5倍以上の高速化 |
| Phase 4 | 成熟度 | PyPIトップ100の30%が基本動作 |

## まとめ

このPythonParserBox実装は、単なる機能追加ではなく、Nyash言語の成熟度を飛躍的に高める戦略的プロジェクト。
エキスパートの指摘を踏まえ、関数単位のフォールバック、Python 3.11固定、意味論の正確な実装、
GIL最小化、テレメトリー重視で着実に実装を進める。