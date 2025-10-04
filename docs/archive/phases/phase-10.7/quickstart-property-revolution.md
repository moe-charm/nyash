# Phase 10.7 × Property System 革命 - 今すぐ始めるクイックスタート

## 🎯 Property System革命により実現可能になったPython→Nyash実行

2025-09-18のProperty System革命により、Python transpilationが飛躍的に実現可能に！

## 🚀 最短実装ルート（3週間で実用レベル）

### Week 1: 基本プロパティ認識
```bash
# プラグイン作成
cd plugins/
cargo new nyash-python-parser-plugin --lib

# 最小依存関係
echo '[dependencies]
pyo3 = { version = "0.22", features = ["auto-initialize"] }
nyash-plugin-sdk = { path = "../../crates/plugin-sdk" }' >> Cargo.toml
```

### Week 2-3: Property System活用コンパイラ
```rust
// src/lib.rs - 最小実装例
use pyo3::prelude::*;

#[pyclass]
pub struct PythonCompilerBox {
    property_classifier: PropertyClassifier,
}

#[pymethods]
impl PythonCompilerBox {
    #[new]
    pub fn new() -> Self {
        Self {
            property_classifier: PropertyClassifier::new(),
        }
    }
    
    pub fn compile_simple(&self, python_code: &str) -> PyResult<String> {
        let ast = self.parse_python(python_code)?;
        let classified = self.property_classifier.classify(ast);
        let nyash_code = self.generate_nyash_with_properties(classified);
        Ok(nyash_code)
    }
}

struct PropertyClassifier;

impl PropertyClassifier {
    fn new() -> Self { Self }
    
    fn classify(&self, ast: PythonAst) -> ClassifiedAst {
        // Phase 1: 基本パターンのみ
        // @property → computed
        // @cached_property → once
        // __init__代入 → stored
        todo!("実装")
    }
}
```

## 🧪 MVP テストケース

### 入力Python
```python
# test_simple.py
class Counter:
    def __init__(self):
        self.value = 0
    
    @property
    def doubled(self):
        return self.value * 2
    
    @functools.cached_property  
    def expensive_result(self):
        return sum(range(1000))
```

### 期待されるNyash出力
```nyash
box Counter {
    value: IntegerBox                                    // stored
    doubled: IntegerBox { me.value * 2 }                // computed  
    once expensive_result: IntegerBox { sum_range(1000) }  // once
    
    birth() {
        me.value = 0
    }
}
```

### 実行テスト
```bash
# transpilation
nyash --pyc test_simple.py -o test_simple.ny

# ネイティブコンパイル
nyash --compile-native test_simple.ny -o test_app

# 実行（CPython依存なし！）
./test_app
```

## 📊 段階的成功指標

### Phase 1 (1週間後)
- ✅ @property, @cached_property認識
- ✅ 基本クラス → box変換
- ✅ 1つのサンプルPythonファイルが動作

### Phase 2 (2週間後)  
- ✅ 継承、メソッド呼び出し対応
- ✅ 10個のサンプルファイル成功
- ✅ 性能測定（CPythonとの比較）

### Phase 3 (3週間後)
- ✅ エラーハンドリング、例外処理
- ✅ 実用的なPythonライブラリ部分対応
- ✅ AOT配布可能なサンプルアプリ

## 🌟 創造的可能性

### ハイブリッド開発
```python
# Python側で開発・デバッグ
@nyash.optimize  # デコレータで高速化指定
def heavy_computation(data):
    return complex_algorithm(data)

# 本番はNyash AOTで配布
```

### リアルタイムtranspilation IDE
- 左: Pythonコード編集
- 右: リアルタイムNyash生成表示  
- 下: 性能比較グラフ

### 教育効果
- Pythonユーザーが自然にNyashを学習
- Property Systemの概念理解促進

## 🎯 今日から始められるアクション

1. **プラグイン skelton作成** (30分)
2. **pyo3でPython AST取得** (2時間)  
3. **@property検出ロジック** (半日)
4. **最初のbox変換** (1日)
5. **テスト実行** (30分)

Property System革命により、この夢が現実になりました！🚀