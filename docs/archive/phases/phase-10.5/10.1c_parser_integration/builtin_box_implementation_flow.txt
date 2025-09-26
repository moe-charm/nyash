# PythonParserBox ビルトインBox実装フロー（エキスパート統合版）
～CPythonパーサー統合とPhase 1実装の具体的な流れ～
更新日: 2025-08-27

## 🎯 全体の実装フロー

### Step 0: Python 3.11固定（エキスパート推奨）
```
- Python 3.11.9を使用（AST安定性確保）
- pyenvまたはpython3.11コマンドで固定
- py_versionとast_formatをJSON IRに必ず含める
```

### Step 1: ビルトインBoxとしての基盤作成  
```
1. src/boxes/python_parser_box/mod.rs を作成
2. BoxBase + BoxCore統一アーキテクチャに準拠
3. PythonParserBoxの基本構造を定義
4. src/boxes/mod.rs に登録
5. テレメトリー基盤を初期から組み込む
```

### Step 2: pyo3統合とCPythonパーサー接続
```
1. Cargo.tomlに pyo3依存関係追加
2. pyo3::prepare_freethreaded_python()で一度だけ初期化
3. ast.parse()へのFFIブリッジ実装
4. JSON中間表現への変換（Python側でJSON生成）
5. GILは最小限に、py.allow_threads()でRust処理
```

### Step 3: Phase 1機能の実装（必須意味論要素）
```
必須要素（Codex先生強調）:
- LEGBスコーピング + locals/freevars
- デフォルト引数の定義時評価
- イテレータプロトコル（for文）
- for/else + while/else
- Python真偽値判定
- 短絡評価（and/or）

実装手順:
1. 関数単位フォールバック戦略の実装
2. 基本的なAST変換（def, if, for, while, return）
3. 式の変換（算術/比較/論理演算子、関数呼び出し）
4. Nyash ASTへの意味論を保ったマッピング
5. Differential Testingフレームワーク
```

## 📝 具体的な実装コード

### 1. ビルトインBox定義（src/boxes/python_parser_box/mod.rs）
```rust
use crate::core::{BoxBase, BoxCore, NyashBox};
use crate::ast;
use pyo3::prelude::*;
use std::sync::{Arc, Mutex};

pub struct PythonParserBox {
    base: BoxBase,
    py_helper: Arc<Mutex<PyHelper>>,  // Python実行環境
}

impl BoxCore for PythonParserBox {
    fn box_id(&self) -> u64 {
        self.base.box_id()
    }
    
    fn parent_type_id(&self) -> Option<std::any::TypeId> {
        self.base.parent_type_id()
    }
    
    fn fmt_box(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PythonParserBox#{}", self.box_id())
    }
}

impl NyashBox for PythonParserBox {
    fn type_name(&self) -> &'static str {
        "PythonParserBox"
    }
    
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(PythonParserBox {
            base: BoxBase::new(),
            py_helper: self.py_helper.clone(),
        })
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
```

### 2. メソッド実装（Phase 1対応）
```rust
// テレメトリー用構造体
#[derive(Default)]
pub struct CompilationTelemetry {
    compiled_functions: Vec<String>,
    fallback_functions: Vec<(String, String, usize)>, // (name, reason, lineno)
    unsupported_nodes: HashMap<String, usize>,  // node_type -> count
}

impl PythonParserBox {
    // コンストラクタ
    pub fn new() -> Result<Self, String> {
        // 一度だけ初期化
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| {
            pyo3::prepare_freethreaded_python();
        });
        
        // Python環境の初期化
        Python::with_gil(|py| {
            // Python 3.11確認
            let version = py.version_info();
            if version.major != 3 || version.minor != 11 {
                return Err(format!("Python 3.11 required, got {}.{}", 
                    version.major, version.minor));
            }
            
            let helper = PyHelper::new(py)?;
            Ok(PythonParserBox {
                base: BoxBase::new(),
                py_helper: Arc::new(Mutex::new(helper)),
            })
        })
    }
    
    // Python code → JSON AST
    pub fn parse_to_json(&self, code: &str) -> Result<String, String> {
        let helper = self.py_helper.lock().unwrap();
        Python::with_gil(|py| {
            helper.parse_to_json(py, code)
        })
    }
    
    // JSON AST → Nyash AST（Phase 1機能のみ）
    pub fn json_to_nyash_ast(&self, json: &str) -> Result<ast::Program, String> {
        let py_ast: Phase1PythonAst = serde_json::from_str(json)
            .map_err(|e| format!("JSON parse error: {}", e))?;
        
        let converter = Phase1Converter::new();
        converter.convert(py_ast)
    }
    
    // 直接実行（関数単位フォールバック）
    pub fn run(&self, code: &str) -> Result<Box<dyn NyashBox>, String> {
        // まずJSON ASTを取得
        let json_ast = self.parse_to_json(code)?;
        let py_ast: serde_json::Value = serde_json::from_str(&json_ast)?;
        
        // モジュール内の各関数をチェック
        let compiler = FunctionCompiler::new();
        let module_result = compiler.compile_module(&py_ast)?;
        
        // テレメトリー出力（環境変数で制御）
        if std::env::var("NYASH_PYTHONPARSER_TELEMETRY").is_ok() {
            compiler.print_telemetry();
        }
        
        // 実行（コンパイル済み関数はMIR、他はCPython）
        module_result.execute()
    }
}
```

### 3. Python側ヘルパー実装
```rust
// Pythonコードを文字列として埋め込み
const PYTHON_HELPER_CODE: &str = r#"
import ast
import json
import sys

# Python 3.11固定チェック
assert sys.version_info[:2] == (3, 11), f"Python 3.11 required, got {sys.version}"

def ast_to_dict(node):
    """Phase 1: 基本的なAST要素のみ変換（エキスパート推奨JSON IR）"""
    
    result = {
        "node_type": node.__class__.__name__,
        "py_version": "3.11",
        "ast_format": "v1"
    }
    
    # 位置情報（エラー診断用）
    if hasattr(node, 'lineno'):
        result['lineno'] = node.lineno
        result['col_offset'] = node.col_offset
        if hasattr(node, 'end_lineno'):
            result['end_lineno'] = node.end_lineno
            result['end_col_offset'] = node.end_col_offset
    if isinstance(node, ast.Module):
        return {
            "type": "Module",
            "body": [ast_to_dict(stmt) for stmt in node.body]
        }
    elif isinstance(node, ast.FunctionDef):
        # 意味論上重要：デフォルト引数情報を保存
        args_info = {
            "args": [arg.arg for arg in node.args.args],
            "defaults": [ast_to_dict(default) for default in node.args.defaults],
            "kwonlyargs": [arg.arg for arg in node.args.kwonlyargs],
            "kw_defaults": [ast_to_dict(d) if d else None for d in node.args.kw_defaults]
        }
        result.update({
            "name": node.name,
            "args": args_info,
            "body": [ast_to_dict(stmt) for stmt in node.body],
            "decorator_list": [],  # Phase 1では未対応
            "support_level": "full"  # コンパイル可能
        })
        return result
    elif isinstance(node, ast.Return):
        return {
            "type": "Return",
            "value": ast_to_dict(node.value) if node.value else None
        }
    elif isinstance(node, ast.BinOp):
        return {
            "type": "BinOp",
            "op": node.op.__class__.__name__,
            "left": ast_to_dict(node.left),
            "right": ast_to_dict(node.right)
        }
    elif isinstance(node, ast.Call):
        return {
            "type": "Call",
            "func": ast_to_dict(node.func),
            "args": [ast_to_dict(arg) for arg in node.args]
        }
    elif isinstance(node, ast.Name):
        return {
            "type": "Name",
            "id": node.id
        }
    elif isinstance(node, ast.Constant):
        return {
            "type": "Constant",
            "value": node.value
        }
    elif isinstance(node, ast.If):
        return {
            "type": "If",
            "test": ast_to_dict(node.test),
            "body": [ast_to_dict(stmt) for stmt in node.body],
            "orelse": [ast_to_dict(stmt) for stmt in node.orelse]
        }
    elif isinstance(node, ast.For):
        # 意味論上重要：for/else構文
        result.update({
            "target": ast_to_dict(node.target),
            "iter": ast_to_dict(node.iter),
            "body": [ast_to_dict(stmt) for stmt in node.body],
            "orelse": [ast_to_dict(stmt) for stmt in node.orelse],  # else節
            "support_level": "full"
        })
        return result
    
    elif isinstance(node, ast.While):
        # 意味論上重要：while/else構文
        result.update({
            "test": ast_to_dict(node.test),
            "body": [ast_to_dict(stmt) for stmt in node.body],
            "orelse": [ast_to_dict(stmt) for stmt in node.orelse],  # else節
            "support_level": "full"
        })
        return result
    
    elif isinstance(node, ast.BoolOp):
        # 意味論上重要：短絡評価
        result.update({
            "op": node.op.__class__.__name__,  # And, Or
            "values": [ast_to_dict(v) for v in node.values],
            "support_level": "full"
        })
        return result
    else:
        # Phase 1では未対応（テレメトリー用）
        return {
            "node_type": "Unsupported", 
            "original_type": type(node).__name__,
            "support_level": "fallback",
            "lineno": getattr(node, 'lineno', -1)
        }

def parse_to_json(code):
    try:
        tree = ast.parse(code)
        return json.dumps(ast_to_dict(tree))
    except Exception as e:
        return json.dumps({"type": "Error", "message": str(e)})
"#;

struct PyHelper {
    // Python側のヘルパー関数への参照を保持
    parse_func: PyObject,
}

impl PyHelper {
    fn new(py: Python) -> PyResult<Self> {
        // ヘルパーコードをPythonで実行
        let helpers = PyModule::from_code(py, PYTHON_HELPER_CODE, "helper.py", "helper")?;
        let parse_func = helpers.getattr("parse_to_json")?.to_object(py);
        
        Ok(PyHelper { parse_func })
    }
    
    fn parse_to_json(&self, py: Python, code: &str) -> Result<String, String> {
        match self.parse_func.call1(py, (code,)) {
            Ok(result) => result.extract::<String>(py)
                .map_err(|e| format!("Extract error: {}", e)),
            Err(e) => Err(format!("Parse error: {}", e))
        }
    }
}
```

### 4. Phase 1 AST変換器
```rust
struct Phase1Converter;

impl Phase1Converter {
    fn new() -> Self {
        Phase1Converter
    }
    
    fn convert(&self, py_ast: Phase1PythonAst) -> Result<ast::Program, String> {
        match py_ast {
            Phase1PythonAst::Module { body } => {
                let mut items = vec![];
                for stmt in body {
                    match self.convert_statement(stmt)? {
                        Some(item) => items.push(item),
                        None => {} // 未対応要素はスキップ
                    }
                }
                Ok(ast::Program { items })
            }
            _ => Err("Expected Module at top level".into())
        }
    }
    
    fn convert_statement(&self, stmt: Phase1PythonAst) -> Result<Option<ast::ProgramItem>, String> {
        match stmt {
            Phase1PythonAst::FunctionDef { name, args, body } => {
                // Python def → Nyash function
                let params = args.into_iter()
                    .map(|arg| ast::Parameter { name: arg, ty: None })
                    .collect();
                
                let nyash_body = self.convert_body(body)?;
                
                Ok(Some(ast::ProgramItem::Function(ast::FunctionDef {
                    name,
                    params,
                    body: nyash_body,
                    return_type: None,
                })))
            }
            Phase1PythonAst::Return { value } => {
                let expr = value
                    .map(|v| self.convert_expression(v))
                    .transpose()?;
                    
                Ok(Some(ast::ProgramItem::Statement(ast::Statement::Return(expr))))
            }
            // 他の文も同様に変換
            _ => Ok(None) // Phase 1では未対応
        }
    }
    
    fn convert_expression(&self, expr: Phase1PythonAst) -> Result<ast::Expression, String> {
        match expr {
            Phase1PythonAst::BinOp { op, left, right } => {
                let left = Box::new(self.convert_expression(*left)?);
                let right = Box::new(self.convert_expression(*right)?);
                
                let op = match op.as_str() {
                    "Add" => ast::BinaryOp::Add,
                    "Sub" => ast::BinaryOp::Sub,
                    "Mul" => ast::BinaryOp::Mul,
                    "Div" => ast::BinaryOp::Div,
                    _ => return Err(format!("Unsupported operator: {}", op))
                };
                
                Ok(ast::Expression::BinaryOp { op, left, right })
            }
            Phase1PythonAst::Constant { value } => {
                // Python定数 → Nyashリテラル
                match value {
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            Ok(ast::Expression::Integer(i))
                        } else if let Some(f) = n.as_f64() {
                            Ok(ast::Expression::Float(f))
                        } else {
                            Err("Unsupported number type".into())
                        }
                    }
                    serde_json::Value::String(s) => {
                        Ok(ast::Expression::String(s))
                    }
                    serde_json::Value::Bool(b) => {
                        Ok(ast::Expression::Bool(b))
                    }
                    _ => Err("Unsupported constant type".into())
                }
            }
            // 他の式も同様
            _ => Err("Unsupported expression in Phase 1".into())
        }
    }
}
```

### 5. インタープリター統合（src/interpreter/builtins.rs）
```rust
// ビルトインBox登録に追加
pub fn register_builtin_boxes(env: &mut Environment) {
    // 既存のBox登録...
    
    // PythonParserBox追加
    env.register_builtin_box("PythonParserBox", || {
        match PythonParserBox::new() {
            Ok(parser) => Arc::new(parser) as Arc<dyn NyashBox>,
            Err(e) => panic!("Failed to initialize PythonParserBox: {}", e)
        }
    });
}
```

### 6. 使用例とテストケース
```nyash
// test_python_parser_phase1.nyash
local py = new PythonParserBox()

// Phase 1: 基本的な関数定義と演算
local code = """
def add(x, y):
    return x + y

def multiply(x, y):
    return x * y

def calculate(a, b):
    sum_val = add(a, b)
    prod_val = multiply(a, b)
    return sum_val + prod_val
"""

// パースしてJSON ASTを確認
local json_ast = py.parse_to_json(code)
print("JSON AST: " + json_ast)

// Nyash ASTに変換
local nyash_ast = py.json_to_nyash_ast(json_ast)
print("Conversion successful!")

// 実行（最初はCPython経由）
local result = py.run(code + "\nprint(calculate(10, 5))")
```

## 📊 段階的な実装計画

### Week 1: 基盤構築
- [ ] PythonParserBoxの基本構造
- [ ] pyo3統合とPython環境初期化
- [ ] parse_to_json基本実装
- [ ] エラーハンドリング

### Week 2: Phase 1変換器
- [ ] Phase1PythonAstの定義
- [ ] Phase1Converterの実装
- [ ] 基本的な文と式の変換
- [ ] テストケース作成

### Week 3: 統合とテスト
- [ ] インタープリター統合
- [ ] CPython exec経由の実行
- [ ] ベンチマーク準備
- [ ] ドキュメント整備

## 🚀 期待される成果

### Phase 1完了時点で実現できること：
1. **基本的なPythonコードの実行**
   - 関数定義、算術演算、条件分岐、ループ
   
2. **Nyash ASTへの変換**
   - 将来のMIR/JIT最適化への道筋
   
3. **統合開発環境**
   - PythonコードとNyashコードの混在実行
   
4. **性能測定基盤**
   - CPython実行 vs Nyash実行の比較

## 📡 テレメトリー出力例

```bash
# 環境変数で制御
export NYASH_PYTHONPARSER_TELEMETRY=1  # 基本統計
export NYASH_PYTHONPARSER_TELEMETRY=2  # 詳細ログ
export NYASH_PYTHONPARSER_STRICT=1     # フォールバック時にパニック

# 実行例
./target/release/nyash test_python_parser.nyash

# 出力
[PythonParser] Module: test.py (Python 3.11)
  Functions: 10 total
  Compiled: 7 (70%) → Nyash MIR/JIT
  Fallback: 3 (30%) → CPython exec
    - async_function: unsupported node 'AsyncFunctionDef' at line 23
    - generator_func: unsupported node 'Yield' at line 45
    - decorator_func: unsupported node 'decorator_list' at line 67
  
  Unsupported Nodes Summary:
    AsyncFunctionDef: 1
    Yield: 2
    ClassDef: 1
```

## 📊 Differential Testingフレームワーク

```rust
// CPythonとNyashの出力比較
pub fn differential_test(code: &str) -> TestResult {
    // CPythonで実行（オラクル）
    let python_result = Python::with_gil(|py| {
        capture_python_execution(py, code)
    })?;
    
    // Nyashで実行
    let nyash_result = execute_with_pythonparser(code)?;
    
    // 結果比較
    compare_results(python_result, nyash_result)
}
```

---
作成日: 2025-08-27
Phase 1実装の具体的な手順とエキスパートフィードバック統合