# PythonParserBox設計提案 - CPythonパーサーを使ったPython→Nyash変換

## 概要
CPythonの公式パーサーを活用して、PythonコードをNyashで直接実行可能にする革命的なアプローチ。
PythonコードをNyashのAST/MIRに変換し、Nyashの最適化・JITコンパイルの恩恵を受けられるようにする。

## アーキテクチャ

### 1. PythonParserBox（ビルトインBox）
```nyash
// 使用例
local py_parser = new PythonParserBox()

// Pythonコードを直接パース
local ast = py_parser.parse("""
def calculate(x, y):
    return x * 2 + y
    
result = calculate(10, 5)
""")

// NyashのASTに変換
local nyash_ast = py_parser.to_nyash_ast(ast)

// 直接実行も可能
local result = py_parser.run(python_code)
```

### 2. 実装構造
```rust
pub struct PythonParserBox {
    base: BoxBase,
    parser: CPythonParser,  // CPythonの公式パーサー使用
}

impl PythonParserBox {
    // Python → Python AST
    pub fn parse(&self, code: &str) -> Box<dyn NyashBox> {
        let py_ast = self.parser.parse_string(code);
        Box::new(PythonAstBox { ast: py_ast })
    }
    
    // Python AST → Nyash AST
    pub fn to_nyash_ast(&self, py_ast: &PythonAstBox) -> Box<dyn NyashBox> {
        let converter = AstConverter::new();
        converter.convert_python_to_nyash(py_ast)
    }
    
    // Python AST → MIR（直接変換）
    pub fn to_mir(&self, py_ast: &PythonAstBox) -> MirModule {
        let mut builder = MirBuilder::new();
        for func in py_ast.functions() {
            self.convert_function_to_mir(&mut builder, func);
        }
        builder.build()
    }
}
```

## AST変換マッピング

### Python → Nyash対応表
| Python AST | Nyash AST | 説明 |
|------------|-----------|------|
| FunctionDef | FunctionDecl | 関数定義 |
| BinOp | BinaryOp | 二項演算 |
| Call | MethodCall | 関数呼び出し |
| Assign | Assignment | 代入 |
| If | IfStatement | 条件分岐 |
| While/For | LoopStatement | ループ |
| Return | ReturnStatement | return文 |
| Import | NewBox/LoadPlugin | import → Box化 |

### 型変換戦略
```rust
// Python動的型 → Nyash Box
match py_value {
    PyInt(n) => IntegerBox::new(n),
    PyFloat(f) => FloatBox::new(f),
    PyStr(s) => StringBox::new(s),
    PyList(items) => ArrayBox::from_iter(items),
    PyDict(map) => MapBox::from_iter(map),
    PyObject(obj) => PythonObjectBox::new(obj),  // 変換不能な場合
}
```

## MIR生成例

### Pythonコード
```python
def calculate(x, y):
    return x * 2 + y
```

### 生成されるMIR
```
function calculate(%x, %y) {
    Load(%1, %x)
    Const(%2, 2)
    BinOp(%3, Mul, %1, %2)
    Load(%4, %y)
    BinOp(%5, Add, %3, %4)
    Return(%5)
}
```

## 利点

1. **完全な互換性**: CPython公式パーサーで100%正確なパース
2. **統一最適化**: PythonコードもNyashのMIR最適化パイプラインを通る
3. **JIT/AOTコンパイル**: PythonコードをネイティブコードにJIT/AOTコンパイル可能
4. **段階的移行**: 既存Pythonコードを徐々にNyashに移行
5. **デバッグ統一**: Nyashのデバッグツールでpythonコードもデバッグ可能

## 実装フェーズ

### Phase 1: 基本パーサー統合
- CPythonパーサーのFFIバインディング
- parse()メソッドでPython ASTを取得
- AST可視化（dump_ast）

### Phase 2: AST変換
- Python AST → Nyash AST変換器
- 基本的な文法要素のサポート
- 型変換システム

### Phase 3: MIR直接生成
- Python AST → MIR変換
- Python特有の最適化（動的型推論等）
- ベンチマーク

### Phase 4: エコシステム統合
- NumPy等の主要ライブラリサポート
- Python例外 → Nyashエラー変換
- async/await対応
- GIL管理の自動化

## 技術的課題と解決策

### 1. GIL（Global Interpreter Lock）
- 解決策: PythonコードはGILスコープ内で実行
- 将来: MIR変換後はGILフリーで実行可能

### 2. Python動的型とNyash Box型のマッピング
- 解決策: 実行時型情報を保持するPythonObjectBox
- 最適化: よく使う型（int, str等）は専用Boxに変換

### 3. Pythonモジュールシステム
- 解決策: importをNyashのプラグインロードにマッピング
- pip packages → Nyashプラグインとして扱う

## 実用例

### 機械学習コードの実行
```nyash
local ml_code = """
import numpy as np
from sklearn.linear_model import LinearRegression

# データ準備
X = np.array([[1, 1], [1, 2], [2, 2], [2, 3]])
y = np.dot(X, np.array([1, 2])) + 3

# モデル訓練
model = LinearRegression()
model.fit(X, y)

# 予測
predictions = model.predict(np.array([[3, 5]]))
print(f'Prediction: {predictions[0]}')
"""

local py_parser = new PythonParserBox()
local result = py_parser.run(ml_code)
```

### Webアプリケーション
```nyash
local flask_app = """
from flask import Flask, jsonify

app = Flask(__name__)

@app.route('/api/hello/<name>')
def hello(name):
    return jsonify({'message': f'Hello, {name}!'})

if __name__ == '__main__':
    app.run(port=5000)
"""

local py_parser = new PythonParserBox()
py_parser.run(flask_app)  // FlaskアプリがNyash内で起動
```

## 期待される効果

1. **パフォーマンス向上**: PythonコードがJITコンパイルされ高速化
2. **メモリ効率**: NyashのGC/メモリ管理を活用
3. **相互運用性**: Python↔Nyash↔Rust↔JS等の自由な組み合わせ
4. **開発効率**: 既存Pythonライブラリをそのまま使える
5. **将来性**: PythonコードをネイティブAOTコンパイル可能

## まとめ

PythonParserBoxは、Pythonの豊富なエコシステムとNyashの高性能実行エンジンを組み合わせる画期的なアプローチ。
CPythonパーサーの信頼性とNyashのMIR/JIT最適化により、Pythonコードをより高速に、より効率的に実行できる。