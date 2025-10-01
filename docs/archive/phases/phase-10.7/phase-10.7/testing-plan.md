# Python Native Testing Plan

## 🎯 テスト戦略の全体像

「世界中のPythonコードがNyashのテストケース」という思想のもと、CPythonをオラクルとして使用する包括的なテスト戦略。

## 🧪 テストレベル

### 1. プラグインレベルテスト

#### PythonParserBox Tests
```rust
// plugins/nyash-python-parser-plugin/tests/parser_tests.rs
#[test]
fn test_parse_simple_function() {
    let parser = create_parser_box();
    let code = "def add(x, y): return x + y";
    let ast = parser.parse(create_string_box(code));
    
    assert_eq!(ast.get_type().to_string(), "Module");
    let functions = ast.get_children();
    assert_eq!(functions.length(), 1);
}

#[test]
fn test_parse_with_telemetry() {
    let parser = create_parser_box();
    parser.enable_telemetry(true);
    
    let code = r#"
def supported(): return 1
async def unsupported(): await foo()
    "#;
    
    parser.parse(create_string_box(code));
    let stats = parser.get_stats();
    
    assert_eq!(stats.get("total_functions"), 2);
    assert_eq!(stats.get("supported_functions"), 1);
}
```

#### PythonCompilerBox Tests
```rust
#[test]
fn test_compile_arithmetic() {
    let compiler = create_compiler_box();
    let ast = /* ... */;
    
    let mir = compiler.compile(ast);
    assert!(mir.is_ok());
    
    // MIR検証
    let module = mir.unwrap();
    assert!(module.has_function("add"));
}
```

### 2. Differential Testing Framework

```nyash
// tests/differential/framework.nyash
box DifferentialTester {
    init { oracle, implementation, results }
    
    constructor() {
        me.oracle = new PythonRuntimeBox()  // CPython
        me.implementation = new NativeEngine()
        me.results = new ArrayBox()
    }
    
    test(code) {
        local oracle_result, impl_result
        
        // CPythonで実行
        oracle_result = me.oracle.eval(code)
        
        // Native実装で実行
        impl_result = me.implementation.exec(code)
        
        // 結果比較
        return me.compare(oracle_result, impl_result)
    }
    
    compare(expected, actual) {
        // 出力、戻り値、例外を比較
        local match = new MapBox()
        match.set("output", expected.output == actual.output)
        match.set("return", expected.return == actual.return)
        match.set("exception", expected.exception == actual.exception)
        return match
    }
}
```

### 3. テストケース生成

#### 基本テストスイート
```python
# tests/suites/phase1_tests.py

# 算術演算
def test_arithmetic():
    assert add(2, 3) == 5
    assert multiply(4, 5) == 20
    assert divide(10, 2) == 5.0  # true division

# 制御フロー
def test_control_flow():
    # if/else
    result = conditional_logic(True, 10, 20)
    assert result == 10
    
    # for/else
    found = search_with_else([1, 2, 3], 5)
    assert found == "not found"  # else節実行

# デフォルト引数の罠
def test_default_args():
    list1 = append_to_default(1)
    list2 = append_to_default(2)
    assert list1 is list2  # 同じリスト！
```

#### Fuzzing with Hypothesis
```python
# tests/fuzzing/property_tests.py
from hypothesis import given, strategies as st

@given(st.integers(), st.integers())
def test_arithmetic_properties(x, y):
    """算術演算の性質をテスト"""
    # Commutativity
    assert add(x, y) == add(y, x)
    
    # Identity
    assert add(x, 0) == x
    
    # Differential testing
    native_result = native_add(x, y)
    cpython_result = x + y
    assert native_result == cpython_result
```

### 4. ベンチマークスイート

```nyash
// benchmarks/numeric_suite.nyash
box NumericBenchmark {
    run() {
        local suite = new BenchmarkSuite()
        
        // Fibonacci
        suite.add("fibonacci", {
            "cpython": { return me.runCPython("fib.py") },
            "native": { return me.runNative("fib.py") }
        })
        
        // Matrix multiplication
        suite.add("matrix_mult", {
            "cpython": { return me.runCPython("matrix.py") },
            "native": { return me.runNative("matrix.py") }
        })
        
        return suite.execute()
    }
}

// 実行結果例
// fibonacci:
//   CPython: 1.234s
//   Native:  0.123s (10.0x faster)
// matrix_mult:
//   CPython: 5.678s
//   Native:  0.456s (12.4x faster)
```

### 5. 回帰テスト

```yaml
# .github/workflows/python-native-tests.yml
name: Python Native Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - name: Differential Tests
        run: |
          cargo test --package nyash-python-parser-plugin
          cargo test --package nyash-python-compiler-plugin
          
      - name: Coverage Report
        run: |
          ./tools/measure_compilation_coverage.sh
          # Expected output:
          # Phase 1 compatible files: 15%
          # Phase 2 functions: 40% compilable
          # Phase 3 functions: 10% compilable
```

## 📊 メトリクス収集

### コンパイル成功率
```nyash
// 自動計測ツール
box CoverageAnalyzer {
    analyze(directory) {
        local parser = new PythonParserBox()
        local compiler = new PythonCompilerBox()
        local stats = new MapBox()
        
        for file in directory.glob("*.py") {
            local ast = parser.parseFile(file)
            local result = compiler.compile(ast)
            
            stats.increment("total")
            if result.isOk() {
                stats.increment("success")
            } else {
                stats.increment("not_compilable")
                stats.record("unsupported", result.getError())
            }
        }
        
        return stats
    }
}
```

### パフォーマンス追跡
```sql
-- メトリクスDB
CREATE TABLE benchmark_results (
    id SERIAL PRIMARY KEY,
    test_name VARCHAR(255),
    implementation VARCHAR(50),  -- 'cpython' or 'native'
    execution_time FLOAT,
    memory_usage BIGINT,
    timestamp TIMESTAMP,
    git_hash VARCHAR(40)
);
```

## 🚨 失敗時の診断

### デバッグ情報収集
```nyash
// コンパイル失敗時の詳細情報
compiler.enableDebug(true)
result = compiler.compile(ast)

if result.isErr() {
    local diag = compiler.getDiagnostics()
    print("Failed at: " + diag.get("location"))
    print("Reason: " + diag.get("reason"))
    print("AST node: " + diag.get("node_type"))
    print("Suggestion: " + diag.get("suggestion"))
}
```

### トレース機能
```
NYASH_PYTHON_TRACE=1 ./target/release/nyash test.py
[Parser] Parsing function 'compute' at line 5
[Compiler] Compiling BinOp: Add at line 7
[Compiler] Unsupported: YieldFrom at line 15
[Error] Cannot compile function 'generator_func' - yield not supported
```

## ✅ 受け入れ基準

### Phase 1完了
- [ ] 基本テストスイート100%パス
- [ ] Differential testing 100%一致
- [ ] Phase 1対応コードの100%コンパイル成功
- [ ] 10x性能向上（数値計算ベンチマーク）

### 各PR必須
- [ ] 新機能の単体テスト
- [ ] Differential testケース追加
- [ ] ベンチマーク結果（該当する場合）
- [ ] カバレッジ低下なし