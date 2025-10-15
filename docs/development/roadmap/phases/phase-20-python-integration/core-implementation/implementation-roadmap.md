# PythonParserBox 実装ロードマップ（エキスパート統合版）
Based on ChatGPT5's Python Language Feature Surface Map + Expert Feedback
更新日: 2025-08-27

## 🎯 実装優先順位の考え方（エキスパート統合）

### 🏯 核心戦略：関数単位フォールバック
**両エキスパートが強調：** ファイル全体ではなく、**関数単位**でコンパイル/フォールバックを判断
```python
def supported_function():   # → Nyash MIR/JIT
    return x + y

def unsupported_function(): # → CPython exec
    yield from generator   # Phase 1では未対応
```

### 🔧 Python 3.11固定
- AST安定性確保（3.8 Constant統一、3.10 match/case、3.12位置情報）
- `py_version`と`ast_format`をJSON IRに埋め込む

### 🌟 Differential Testing戦略
- **世界中のPythonコードがNyashのテストケースに**
- CPythonをオラクルとして使用、出力・戻り値・例外を比較
- 微妙なセマンティクスバグを自動発見

### 📊 テレメトリー重視
- 未対応ノードの記録（`support_level`フィールド）
- フォールバック率の計測
- ソース位置情報保持（`lineno/col_offset/end_*`）

## 📋 Phase 1: Core Subset（1-2週間）
**目標**: 基本的なPythonコードをNyashで実行可能にする

### ❌ Phase 1での必須意味論要素（Codex先生強調）
- **LEGB + locals/freevars**: スコーピング規則
- **デフォルト引数の評価タイミング**: 定義時に一度だけ
- **イテレータベースのfor文**: `__iter__`/`__next__`プロトコル
- **for/else + while/else**: Python独特のelse節
- **Python真偽値判定**: `__bool__` → `__len__`
- **短絡評価**: and/orの正確な挙動

### 文（Statement）
- [x] def - 関数定義 → Nyash関数/Box
  - デフォルト引数の定義時評価
  - argumentsオブジェクトの完全解析
- [x] if/elif/else - 条件分岐 → CondBr
- [x] for - ループ → Loop + Iterator
  - **else節対応必須**
- [x] while - ループ → Loop  
  - **else節対応必須**
- [x] break/continue - ループ制御
- [x] return - 戻り値 → Return
- [ ] pass - 空文
- [ ] import（Phase 3へ延期）

### 式（Expression）  
- [x] 関数呼び出し - Call → BoxCall
- [x] 算術演算子 - +,-,*,/,//,% → BinOp
  - `/`: true division（常にfloat）
  - `//`: floor division
- [x] 比較演算子 - ==,!=,<,>,<=,>=,is,is not → Compare
- [x] 論理演算子 - and,or,not → BoolOp/UnaryOp
  - 短絡評価の正確な実装
- [x] 変数参照/代入 - Name → Load/Store
- [x] リテラル - 数値/文字列/bool → Constant
- [x] 三項演算子 - IfExp

### データ型（最小限）
- [x] int → IntegerBox（大整数対応）
- [x] float → FloatBox（NaNの扱い注意）  
- [x] str → StringBox
- [x] bool → BoolBox
- [x] list（基本） → ArrayBox

## 📋 Phase 2: Data Model（2-3週間）
**目標**: Pythonの特殊メソッドをNyashのBoxメソッドにマッピング

### 特殊メソッド
- [ ] __init__ → constructor/birth
- [ ] __len__ → length()
- [ ] __getitem__ → get()
- [ ] __setitem__ → set()
- [ ] __iter__ → iterator()
- [ ] __str__ → toString()

### コレクション拡張
- [ ] dict → MapBox
- [ ] tuple → ImmutableArrayBox（新規）
- [ ] set → SetBox（新規）

### 演算子オーバーロード
- [ ] __add__, __sub__ 等 → operator+, operator-
- [ ] __eq__, __lt__ 等 → equals(), compareTo()

## 📋 Phase 3: Advanced Features（1ヶ月）
**目標**: Pythonの生産性の高い機能を実装

### 制御フロー拡張
- [ ] try/except → エラーハンドリング
- [ ] with文 → リソース管理
- [ ] break/continue → ループ制御

### 高度な機能
- [ ] ジェネレータ（yield） → GeneratorBox
- [ ] デコレータ → 関数ラッパー
- [ ] 内包表記 → 最適化されたループ
- [ ] ラムダ式 → 匿名関数

### クラスシステム
- [ ] class文 → box定義
- [ ] 継承 → from構文
- [ ] super() → from Parent.method()

## 📋 Phase 4: Modern Python（将来）
**目標**: 最新のPython機能をサポート

### 非同期
- [ ] async/await → 非同期Box（将来のNyash非同期と統合）
- [ ] async for/with → 非同期イテレータ

### パターンマッチ（3.10+）
- [ ] match/case → Nyashのパターンマッチ（将来実装時）

### 型ヒント
- [ ] 型アノテーション → MIRの型情報として活用
- [ ] typing モジュール → 静的型チェック情報

## 🚀 実装戦略

### Step 1: AST変換の基礎（Phase 1開始）
```python
# Python側でAST→JSON
import ast
import json

def parse_to_json(code):
    tree = ast.parse(code)
    return json.dumps(ast_to_dict(tree))

# 最小限のノードから実装
def ast_to_dict(node):
    if isinstance(node, ast.FunctionDef):
        return {
            "type": "FunctionDef",
            "name": node.name,
            "args": [arg.arg for arg in node.args.args],
            "body": [ast_to_dict(stmt) for stmt in node.body]
        }
    # ... 他のノードタイプを順次追加
```

### Step 2: Nyash AST生成（Rust側）
```rust
// JSON → Nyash AST
fn convert_python_ast(json: &str) -> Result<ast::Program> {
    let py_ast: PythonAst = serde_json::from_str(json)?;
    match py_ast {
        PythonAst::FunctionDef { name, args, body } => {
            // Python def → Nyash function
            ast::BoxDef {
                name,
                methods: vec![ast::Method {
                    name: name.clone(),
                    params: args,
                    body: convert_statements(body),
                }],
                ..
            }
        }
        // ... 他のケース
    }
}
```

### Step 3: 段階的な実行
1. 最初はCPython exec()でそのまま実行
2. 変換可能な部分からMIR生成
3. MIR化された部分はVM/JITで高速実行
4. 未対応部分は自動的にCPythonフォールバック

## 📊 期待される成果

### Phase 1完了時点
- 簡単な数値計算スクリプトが2-5倍高速化
- 基本的なループが最適化される
- Nyashの既存Box（FileBox等）がPythonから使える

### Phase 2完了時点
- Pythonのリスト/辞書操作が高速化
- NyashとPythonのデータ構造が相互運用可能
- 特殊メソッドによる自然な統合

### Phase 3完了時点
- Pythonの生産的な機能がNyashで高速実行
- 既存Pythonコードの大部分が動作
- デコレータやジェネレータも最適化

## 🎯 最初の一歩（今すぐ開始）

1. pyo3でPythonParserBoxの骨組み作成
2. 最小限のparse_to_json実装（def + return）
3. 単純な関数のAST変換テスト
4. "Hello from Python in Nyash"を表示

```python
# 最初のテストケース
def hello():
    return "Hello from Python in Nyash"

# これがNyashで動けば成功！
```

## 📊 成功の測定基準（エキスパート推奨）

### 定量的指標
| 指標 | 目標 | 測定方法 |
|------|-------|----------|
| カバレッジ率 | 70%以上 | コンパイル済み vs フォールバック関数の比率 |
| 性能向上 | 2-10倍 | 純Pythonループのベンチマーク |
| バグ発見数 | 10+件/Phase | Differential Testingで発見されたNyashバグ |
| エコシステム | 1以上 | 動作する有名Pythonライブラリ |

### マイルストーン
- Phase 1: "Hello from Python in Nyash"が動作
- Phase 2: scikit-learnの基本アルゴリズムが動作
- Phase 3: FlaskのHello Worldが動作
- Phase 4: PyPIトップ100の30%が基本動作

## 🔧 GIL管理の黄金律

```rust
// GILは最小限に！
let json_ast = Python::with_gil(|py| {
    // Python側でJSON生成（高速）
    py_helper.parse_to_json(py, code)
})?;

// GIL外でRust処理（並列可能）
let nyash_ast = py.allow_threads(|| {
    convert_json_to_nyash(json_ast)
});
```

## 🔍 JSON IR設計（Codex先生推奨）

```json
{
    "node_type": "FunctionDef",
    "py_version": "3.11",
    "ast_format": "v2",
    "support_level": "full",  // "partial", "fallback"
    "lineno": 1,
    "col_offset": 0,
    "end_lineno": 3,
    "end_col_offset": 15,
    "name": "hello",
    "args": {...},
    "body": [...]
}
```

---
作成日: 2025-08-27
ChatGPT5のサーフェスマップ + Gemini/Codex先生のエキスパートフィードバックを統合