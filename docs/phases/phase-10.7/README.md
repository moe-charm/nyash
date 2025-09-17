# Phase 10.7 - True Python Native via Plugin Boxes

## 🎯 概要

PythonコードをNyashで**本当にネイティブ実行**する。CPythonへの依存なしに、Pythonコードが完全にNyash MIR/JIT経由で機械語として実行される。

### 現状 vs 理想

**現状（Phase 10.6）**: PyRuntimeBox → libpython呼び出し
**理想（Phase 10.7）**: Python → Nyashスクリプト → MIR → ネイティブ

## 🏗️ アーキテクチャ：トランスパイル方式

```
Python AST → CorePy IR → Nyash AST → Nyashスクリプト
```

### なぜトランスパイル？

1. **透明性**: 生成コードが見える・デバッグできる・手を加えられる
2. **既存資産活用**: Nyashコンパイラの最適化を自動享受
3. **教育的価値**: PythonとNyashの対応が学習価値を持つ
4. **段階的改善**: 生成コードの品質を徐々に向上

### プラグインBox群

- **PythonParserBox**: Python → AST変換
- **PythonCompilerBox**: AST → Nyashスクリプト生成
- **py_runtime.ny**: Pythonセマンティクス保持ライブラリ

## ⚠️ All or Nothing設計（フォールバックなし）

**コンパイルできる or できない の2択のみ**

```nyash
compiler = new PythonCompilerBox()
result = compiler.compile(ast)

if result.isOk() {
    // 100%コンパイル成功 → ネイティブ実行
    print("Success! Native execution ready.")
} else {
    // 未対応機能あり → 完全拒否
    print("Cannot compile: " + result.getError())
    print("Use PyRuntimeBox instead.")
}
```

理由：開発時と本番時で挙動が変わるのは最悪の設計

## 📋 実装フェーズ

### Phase 10.7a - Parser Plugin（1週間）
- PythonParserBoxの実装
- Python AST → ASTBox変換
- テレメトリー基盤

### Phase 10.7b - Compiler Core（2週間）
**Phase 1機能（必須）**
- 関数定義、条件分岐、ループ
- 演算子、関数呼び出し
- Python固有：LEGB、デフォルト引数、for/else

### Phase 10.7c - Validation & Testing（1週間）
- コンパイル可能性の事前検証
- Differential testing（CPythonと比較）
- 明確なエラーメッセージ

### Phase 10.7d - Coverage拡大（3-4週間）
**Phase 2**: 例外処理、with文、comprehensions
**Phase 3**: async/await、デコレータ、ジェネレータ

## 🧪 py_runtime設計

```nyash
// Pythonセマンティクスを忠実に再現
box PyRuntime {
    py_truthy(x) {
        // Python的真偽値判定
        if x == null or x == false { return false }
        if x.hasMethod("__bool__") { return x.__bool__() }
        if x.hasMethod("__len__") { return x.__len__() != 0 }
        return true
    }
    
    py_getattr(obj, name) {
        // ディスクリプタプロトコル、MRO探索
    }
    
    py_call(f, args, kwargs) {
        // デフォルト引数、*args/**kwargs処理
    }
}
```

## 📊 成功指標

### Phase 1完了時
```
Compilable files: 15/100 (15%)
Performance (numeric): 10x faster than CPython
Correctness: 100% (differential testing)
```

### 最終目標（Phase 3）
```
Coverage: 95%+ of common patterns
Performance: 5-20x faster
Distribution: Single binary, no CPython
```

## 🚀 クイックスタート

```bash
# プラグイン作成
cd plugins/
cargo new nyash-python-parser-plugin --lib

# 最小実装
[dependencies]
pyo3 = { version = "0.22", features = ["auto-initialize"] }
nyash-plugin-sdk = { path = "../../crates/plugin-sdk" }

# テスト実行
cargo build --release
../../target/release/nyash test_parser.nyash
```

## 💡 創造的可能性

### ハイブリッドプログラミング
```python
@nyash.vectorize  # PythonデコレータがNyashのSIMD生成！
def matrix_multiply(a, b):
    return a @ b
```

### 言語の共進化
- Nyashが「Pythonで最も使われるイディオム」から学習
- Pythonに「Nyash-aware」コーディングスタイル誕生

### 教育的インパクト
左にPython、右にリアルタイムNyash変換のPlayground

## 📚 参考資料

- **archive/gemini-analysis-transpile-beauty.md** - 創造性分析
- **archive/codex-analysis-technical-implementation.md** - 技術分析