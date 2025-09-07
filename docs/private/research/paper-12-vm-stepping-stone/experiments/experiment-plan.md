# 実験計画 - VM Stepping Stone論文

## 実験1: 開発効率の定量化

### 1.1 コミット履歴分析
```bash
# MIR実装期間
git log --after="2024-08-01" --before="2024-09-01" --oneline | grep -i mir | wc -l

# VM実装期間  
git log --after="2024-09-01" --before="2024-11-01" --oneline | grep -i vm | wc -l

# LLVM実装期間
git log --after="2024-11-01" --before="2025-02-01" --oneline | grep -i llvm | wc -l
```

### 1.2 コード再利用率測定
- VM実装のコード行数
- LLVMで再利用されたパターン数
- 共通FFI/ABIコードの割合

## 実験2: デバッグ効率の測定

### 2.1 バグ修正時間の比較
- VM実装時のバグ修正時間（Issue解決までの時間）
- LLVM実装時のバグ修正時間（VM比較による短縮効果）

### 2.2 具体例: VInvokeバグ
- 問題: argc処理でLLVMが-1していた
- 解決時間: VMとの比較で即座に特定（約30分）
- 従来手法での推定時間: 2-3時間

## 実験3: パフォーマンス測定

### 3.1 ベンチマークプログラム
```nyash
// fibonacci.nyash
static box Main {
    fib(n) {
        if n < 2 {
            return n
        }
        return me.fib(n - 1) + me.fib(n - 2)
    }
    
    main() {
        return me.fib(35)
    }
}
```

### 3.2 測定項目
- Interpreter実行時間
- VM実行時間
- LLVM実行時間
- メモリ使用量

### 3.3 プラグイン呼び出しオーバーヘッド
```nyash
// plugin_bench.nyash
static box Main {
    main() {
        local m = new MapBox()
        local i = 0
        loop(i < 1000000) {
            m.set(i, i * 2)
            local v = m.get(i)
            i = i + 1
        }
        return 0
    }
}
```

## 実験4: コード品質分析

### 4.1 重複コードの削減
- VM実装での重複コード
- LLVM実装での削減量

### 4.2 アーキテクチャの一貫性
- プラグインABIの統一度
- エラー処理の一貫性

## 実験5: 一般化可能性の検証

### 5.1 他言語への適用シミュレーション
- 小規模言語での検証
- 必要な前提条件の洗い出し

### 5.2 適用可能な言語の特徴
- プラグインシステムを持つ
- 統一的な型システム
- FFI/ABIが重要な役割を持つ

## データ収集スクリプト

### collect_metrics.sh
```bash
#!/bin/bash
# 開発メトリクス収集スクリプト

echo "=== Development Timeline ==="
echo "MIR commits:"
git log --after="2024-08-01" --before="2024-09-01" --oneline | grep -iE "(mir|MIR)" | wc -l

echo "VM commits:"
git log --after="2024-09-01" --before="2024-11-01" --oneline | grep -iE "(vm|VM)" | wc -l

echo "LLVM commits:"
git log --after="2024-11-01" --before="2025-02-01" --oneline | grep -iE "(llvm|LLVM)" | wc -l

echo "=== Code Statistics ==="
echo "VM implementation:"
find src/backend/vm* -name "*.rs" -exec wc -l {} + | tail -1

echo "LLVM implementation:"
find src/backend/llvm* -name "*.rs" -exec wc -l {} + | tail -1

echo "Shared runtime:"
find src/runtime -name "*.rs" -exec wc -l {} + | tail -1
```

## 期待される結果

1. **開発時間の短縮**: VM経由により全体の開発時間が30-40%短縮
2. **バグ修正の高速化**: デバッグ時間が平均60-70%削減
3. **コード品質向上**: 再利用により一貫性のあるコードベース
4. **性能**: LLVMはVMの10-50倍高速（プログラムによる）