[Archived] 旧10.1系ドキュメントです。最新は ../INDEX.md を参照してください。

# Phase 10.1f - テストとベンチマーク

## 🎯 このフェーズの目的
Differential Testingでバグを発見し、性能向上を検証する。

## 🧪 Differential Testing戦略

### 1. テストフレームワーク
```rust
pub fn differential_test(code: &str) -> TestResult {
    // CPythonで実行（オラクル）
    let python_result = capture_python_execution(code)?;
    
    // Nyashで実行
    let nyash_result = execute_with_pythonparser(code)?;
    
    // 結果比較
    compare_results(python_result, nyash_result)
}
```

### 2. 比較項目
- **標準出力** - print文の結果
- **戻り値** - 関数の返す値
- **例外** - エラーメッセージ（正規化後）
- **副作用** - グローバル変数の変更等

### 3. テストコーパス
```
test_corpus/
├── basic/          # 基本構文テスト
├── stdlib/         # 標準ライブラリから抜粋
├── pypi_top100/    # 人気ライブラリから抜粋
└── edge_cases/     # エッジケース集
```

## 📊 ベンチマーク

### 1. 性能測定対象
```python
# 数値計算ベンチマーク
def mandelbrot(max_iter=100):
    # フラクタル計算
    pass

# ループベンチマーク  
def sum_of_primes(n):
    # 素数の和
    pass

# 再帰ベンチマーク
def ackermann(m, n):
    # アッカーマン関数
    pass
```

### 2. 測定項目
- **実行時間** - CPython vs Nyash
- **メモリ使用量** - 最大/平均
- **コンパイル時間** - AST変換時間
- **フォールバック率** - 関数別統計

## 🐛 バグ発見と報告

### 発見されたバグの例
```
[BUG-001] for/else semantics mismatch
  Python: else executed when no break
  Nyash: else never executed
  Fixed in: commit abc123

[BUG-002] Division operator difference  
  Python: 5/2 = 2.5 (float)
  Nyash: 5/2 = 2 (integer)
  Fixed in: commit def456
```

## ✅ 完了条件
- [ ] Differential Testingフレームワークが動作する
- [ ] 基本的なテストコーパスが準備されている
- [ ] 10個以上のバグを発見・修正
- [ ] ベンチマークで2倍以上の高速化を確認
- [ ] CI/CDパイプラインに統合されている

## 📈 成功の測定
- **カバレッジ率**: 70%以上の関数がコンパイル
- **性能向上**: 純Pythonループで2-10倍
- **バグ発見数**: Phase毎に10件以上
- **テスト成功率**: 95%以上

## ⏭️ 次のフェーズ
→ Phase 10.1g (ドキュメント作成)