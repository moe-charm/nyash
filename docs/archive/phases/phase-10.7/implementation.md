# Phase 10.7 実装詳細

## 🛠️ 技術アーキテクチャ

### 2段階変換パイプライン

```
Python AST → CorePy IR → Nyash AST → Nyashスクリプト
```

**CorePy IR**の役割：
- Pythonの複雑な構文を正規化
- セマンティクスを明示的に（with→try/finally等）
- 最適化しやすい中間表現

### 実装構造

```rust
// plugins/nyash-python-parser-plugin/src/lib.rs
#[plugin_box]
pub struct PythonParserBox {
    base: BoxBase,
}

#[plugin_methods]
impl PythonParserBox {
    pub fn parse(&self, code: &str) -> Result<Box<dyn NyashBox>> {
        Python::with_gil(|py| {
            let ast_mod = py.import("ast")?;
            let tree = ast_mod.call_method1("parse", (code,))?;
            Ok(self.convert_ast(tree)?)
        })
    }
}
```

## 📐 Python固有機能の実装戦略

### 1. デフォルト引数の罠

```python
# Python: 定義時に一度だけ評価
def bad_default(lst=[]):
    lst.append(1)
    return lst
```

```nyash
// 生成されるNyash
box GeneratedModule {
    init { _default_lst }
    
    constructor() {
        me._default_lst = new ArrayBox()  // 定義時に一度だけ
    }
    
    bad_default(lst) {
        if lst == null {
            lst = me._default_lst  // 同じインスタンスを再利用！
        }
        lst.append(1)
        return lst
    }
}
```

### 2. LEGB スコーピング

```python
# Local → Enclosing → Global → Builtin
global_var = 1
def outer():
    enclosing_var = 2
    def inner():
        local_var = 3
```

実装：
- シンボルテーブルでスコープ管理
- クロージャはBox/Cellで実装
- global/nonlocalフラグを追跡

### 3. for/else, while/else

```python
for i in range(10):
    if i == 5:
        break
else:
    print("No break")
```

```nyash
// 生成されるNyash
local _broken = false
local _iter = py_iter(range(10))
loop(true) {
    local _next = py_next(_iter)
    if _next.isStopIteration() { break }
    local i = _next.value
    
    if i == 5 {
        _broken = true
        break
    }
}
if not _broken {
    print("No break")
}
```

## 🔧 パスパイプライン

```
Parse Python AST
    ↓
Symbol table analysis
    ↓
Normalize to CorePy IR
    ↓
Scope/closure analysis
    ↓
Type metadata attachment
    ↓
Lower to Nyash AST
    ↓
Peephole optimization
    ↓
Pretty-print + source map
```

## 📊 最適化戦略

### トランスパイル時の最適化
- 定数畳み込み
- ループ不変式の巻き上げ
- ビルトイン関数の直接呼び出し（シャドウイングなし時）
- 冗長な`py_truthy()`の除去

### Nyashコンパイラに委ねる最適化
- インライン展開
- レジスタ割り当て
- ループアンローリング
- ベクトル化

### 型情報の活用
```python
def add(x: int, y: int) -> int:
    return x + y
```
→ 型ヒントがあれば`py_binop`ではなく直接整数演算

## 🐛 エラー処理とデバッグ

### ソースマップ
```json
{
  "version": 3,
  "sources": ["example.py"],
  "mappings": "AAAA,IAAM,CAAC,GAAG...",
  "names": ["add", "x", "y"]
}
```

### デバッグモード
```bash
nyash-transpile --debug example.py
# 出力：
# - CorePy IRダンプ
# - Nyashプレビュー（元のPython行ヒント付き）
# - 変換トレース
```

### エラーメッセージ
```
ERROR: Cannot compile function 'async_func' at line 10
Reason: async/await not supported in Phase 1
AST Node: AsyncFunctionDef
Suggestion: Use PyRuntimeBox or wait for Phase 3
```

## ⚡ パフォーマンス最適化

### ホットパス識別
```nyash
// プロファイル情報を活用
if compiler.isHotPath(func) {
    // 積極的な最適化
    result = compiler.optimizeAggressive(func)
} else {
    // 標準的な変換
    result = compiler.compile(func)
}
```

### JIT連携
```nyash
// 型特化コード生成
@jit_specialize(int, int)
def add(x, y):
    return x + y
```

## 🔌 プラグインAPI

### 変換フック
```rust
trait TransformHook {
    fn before_lower(&mut self, node: &CorePyNode);
    fn after_lower(&mut self, node: &NyashNode);
    fn on_function(&mut self, func: &FunctionDef);
}
```

### カスタムルール
```yaml
# custom_rules.yaml
rules:
  - pattern: "dataclass"
    action: "convert_to_nyash_box"
  - pattern: "numpy.array"
    action: "use_native_array"
```

## 📋 実装チェックリスト

### Phase 1（必須）
- [ ] 関数定義（def）
- [ ] 条件分岐（if/elif/else）
- [ ] ループ（for/while with else）
- [ ] 基本演算子
- [ ] 関数呼び出し
- [ ] return/break/continue
- [ ] LEGB スコーピング
- [ ] デフォルト引数

### Phase 2（拡張）
- [ ] 例外処理（try/except/finally）
- [ ] with文
- [ ] list/dict/set comprehensions
- [ ] lambda式
- [ ] *args, **kwargs

### Phase 3（高度）
- [ ] async/await
- [ ] yield/yield from
- [ ] デコレータ
- [ ] クラス定義（基本）
- [ ] import文