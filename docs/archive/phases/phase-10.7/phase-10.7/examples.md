# Python Native実装例

## 🎯 実装イメージ

### 使用例1: 基本的な関数のネイティブ化

```nyash
// example1_basic.nyash
// Pythonコードをネイティブコンパイル

// Step 1: Pythonコードを用意
code = """
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)

def factorial(n):
    if n == 0:
        return 1
    return n * factorial(n-1)
"""

// Step 2: パース
parser = new PythonParserBox()
ast = parser.parse(code)
print("Parsed functions: " + parser.getStats().get("functions"))

// Step 3: コンパイル
compiler = new PythonCompilerBox()
mir_module = compiler.compile(ast)

// Step 4: 実行
if mir_module.isOk() {
    // ネイティブ実行！
    module = mir_module.get()
    
    // 関数を取得して実行
    fib = module.getFunction("fibonacci")
    result = fib.call(10)
    print("fibonacci(10) = " + result)  // 55
    
    fact = module.getFunction("factorial")  
    result = fact.call(5)
    print("factorial(5) = " + result)   // 120
} else {
    print("Compilation failed: " + mir_module.getError())
}
```

### 使用例2: コンパイル可否の明確な判定

```nyash
// example2_clear_separation.nyash
// コンパイルできるかどうか事前に判定

// Phase 1対応のコード
code_phase1 = """
def compute_sum(n):
    total = 0
    for i in range(n):
        total += i * i
    return total

def factorial(n):
    if n == 0:
        return 1
    return n * factorial(n-1)
"""

// Phase 1未対応のコード
code_unsupported = """
def fibonacci_generator(n):
    a, b = 0, 1
    for _ in range(n):
        yield a
        a, b = b, a + b
"""

// コンパイラーで判定
parser = new PythonParserBox()
compiler = new PythonCompilerBox()

// Phase 1対応コードのチェック
ast1 = parser.parse(code_phase1)
result1 = compiler.compile(ast1)
if result1.isOk() {
    print("✅ Phase 1 code compiled successfully!")
    module = result1.get()
    print("compute_sum(100) = " + module.call("compute_sum", 100))
} else {
    print("❌ Compilation failed: " + result1.getError())
}

// 未対応コードのチェック
ast2 = parser.parse(code_unsupported)
result2 = compiler.compile(ast2)
if result2.isOk() {
    print("✅ Compiled successfully!")
} else {
    print("❌ Cannot compile: " + result2.getError())
    print("   Reason: yield expression not supported in Phase 1")
    print("   Please use PyRuntimeBox instead")
}
```

### 使用例3: プログレッシブ最適化

```nyash
// example3_progressive.nyash
// 実行しながら徐々に最適化

// 型推論付きコンパイラー
compiler = new PythonCompilerBox()
compiler.enableTypeInference(true)
compiler.enableProfiling(true)

// 初回実行（型情報収集）
code = """
def matrix_multiply(A, B):
    # 最初は型が不明
    result = []
    for i in range(len(A)):
        row = []
        for j in range(len(B[0])):
            sum = 0
            for k in range(len(B)):
                sum += A[i][k] * B[k][j]
            row.append(sum)
        result.append(row)
    return result
"""

// プロファイル付き実行
for i in range(5) {
    mir = compiler.compile(parser.parse(code))
    
    // 実行してプロファイル収集
    module = mir.get()
    A = [[1, 2], [3, 4]]
    B = [[5, 6], [7, 8]]
    result = module.call("matrix_multiply", A, B)
    
    // 型情報が蓄積される
    print("Iteration " + i + ": ")
    print("  Type confidence: " + compiler.getTypeConfidence())
    print("  Optimization level: " + compiler.getOptimizationLevel())
}

// 5回実行後、完全に最適化されたコードが生成される
```

### 使用例4: 言語間相互運用

```nyash
// example4_interop.nyash  
// PythonコードとNyashコードのシームレスな連携

// Pythonで数値計算関数を定義
python_math = """
import math

def distance(x1, y1, x2, y2):
    return math.sqrt((x2-x1)**2 + (y2-y1)**2)

def normalize(vector):
    magnitude = math.sqrt(sum(x**2 for x in vector))
    return [x/magnitude for x in vector]
"""

// コンパイルしてNyashから使用
module = compile_python(python_math)

// Nyash側のゲームロジック
box GameObject {
    init { x, y, vx, vy }
    
    update(dt) {
        // Python関数をネイティブ速度で呼び出し
        me.x = me.x + me.vx * dt
        me.y = me.y + me.vy * dt
        
        // 正規化（Pythonの関数を使用）
        local normalized = module.normalize([me.vx, me.vy])
        me.vx = normalized[0]
        me.vy = normalized[1]
    }
    
    distanceTo(other) {
        // Pythonの距離計算関数を使用
        return module.distance(me.x, me.y, other.x, other.y)
    }
}

// 完全にネイティブコードとして実行される！
```

### 使用例5: デバッグとプロファイリング

```nyash
// example5_debug.nyash
// 開発時のデバッグ支援

// デバッグモード有効
parser = new PythonParserBox()
parser.enableDebug(true)

compiler = new PythonCompilerBox()
compiler.enableDebug(true)
compiler.enableSourceMap(true)  // 元のPythonコードへのマッピング

problematic_code = """
def buggy_function(items):
    total = 0
    for item in items:
        # バグ: itemが文字列の場合エラー
        total += item * 2
    return total / len(items)
"""

// コンパイル試行
result = compiler.compile(parser.parse(problematic_code))

if result.isErr() {
    // 詳細なエラー情報
    diag = compiler.getDiagnostics()
    print("Compilation failed at line " + diag.line)
    print("Issue: " + diag.message)
    print("Suggestion: " + diag.suggestion)
    
    // フォールバックで実行してランタイムエラーを確認
    runtime = new PythonRuntimeBox()
    try {
        runtime.exec(problematic_code)
        runtime.call("buggy_function", ["a", "b", "c"])
    } catch (e) {
        print("Runtime error: " + e.message)
        print("This would have been caught at compile time!")
    }
}

// プロファイリング情報
profiler = new PythonProfiler()
profiler.attach(module)
profiler.run()

print("Hot spots:")
print(profiler.getHotSpots())
print("Type instability:")
print(profiler.getTypeInstability())
```

## 🎯 実装の進化

### Phase 1（現在）
```python
# これらがネイティブ化可能
def add(x, y): return x + y
def factorial(n): ...
def fibonacci(n): ...
```

### Phase 2（予定）
```python
# 特殊メソッド対応
class Vector:
    def __add__(self, other): ...
    def __len__(self): ...
    
# 内包表記
squares = [x**2 for x in range(10)]
```

### Phase 3（将来）
```python
# 完全な言語機能
async def fetch_data(): ...
@decorator
def enhanced_function(): ...
yield from generator
```

## 🚀 パフォーマンス期待値

```
Benchmark: Fibonacci(30)
CPython:     1.234s
PyPy:        0.123s  
Nyash Native: 0.012s  (100x faster!)

Benchmark: Matrix Multiplication (100x100)
CPython:     5.678s
NumPy:       0.234s
Nyash Native: 0.198s  (NumPyに匹敵!)
```