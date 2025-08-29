# JIT論文評価方法論

## 評価の3本柱（Gemini先生提案）

### 1. 性能評価
- **ベースライン**: インタープリター、VM（JITなし）
- **比較対象**: GraalVM/Truffle、V8（可能な範囲で）
- **測定項目**:
  - スループット（ops/sec）
  - レイテンシ（関数呼び出し）
  - メモリ使用量
  - コンパイル時間

### 2. 回復力評価
- **故障注入実験**:
  - JIT内部での意図的panic
  - メモリ不足状況
  - 無限ループ検出
- **測定項目**:
  - フォールバック成功率
  - 復旧時間
  - 状態の一貫性

### 3. モジュール性評価
- **拡張性テスト**:
  - 新しい型の追加（Complex、Decimal等）
  - バックエンド交換（Cranelift→LLVM）
  - 新しい最適化パスの追加
- **測定項目**:
  - 変更必要行数
  - インターフェース安定性
  - ビルド時間への影響

## 実験計画

### Phase 1: マイクロベンチマーク（実施済み）
✅ 基本演算（整数、浮動小数点）
✅ 制御フロー（分岐、ループ）
✅ PHIノード（値の合流）

### Phase 2: アプリケーションベンチマーク（計画中）
- [ ] フィボナッチ数列（再帰 vs ループ）
- [ ] 素数判定（計算集約型）
- [ ] ソートアルゴリズム（配列操作）
- [ ] 簡易インタープリター（複雑な制御）

### Phase 3: ストレステスト（計画中）
- [ ] 長時間実行（メモリリーク検証）
- [ ] 並行実行（マルチスレッド環境）
- [ ] エラー注入（ランダムfault）

### Phase 4: 比較評価（計画中）
- [ ] Node.js相当ベンチマーク移植
- [ ] Python相当ベンチマーク移植
- [ ] 実行時間・メモリ使用量比較

## データ収集自動化

### ベンチマークハーネス
```nyash
// benchmark_suite.nyash
static box BenchmarkRunner {
    init { timer, results }
    
    constructor() {
        me.timer = new TimerBox()
        me.results = new MapBox()
    }
    
    runBenchmark(name, func, iterations) {
        local start = me.timer.now()
        
        loop(i < iterations) {
            func()
            i = i + 1
        }
        
        local elapsed = me.timer.now() - start
        me.results.set(name, elapsed / iterations)
    }
    
    exportResults() {
        return me.results.toJson()
    }
}
```

### 統計収集スクリプト
```bash
#!/bin/bash
# collect_stats.sh

echo "=== Nyash JIT Benchmark Suite ==="

# 環境変数設定
export NYASH_JIT_EXEC=1
export NYASH_JIT_THRESHOLD=1
export NYASH_JIT_STATS_JSON=1

# 各ベンチマーク実行
for bench in benchmarks/*.nyash; do
    echo "Running: $bench"
    ./target/release/nyash --backend vm "$bench" > "results/$(basename $bench .nyash).json"
done

# 結果集計
python3 aggregate_results.py results/*.json > final_report.md
```

## 再現性の確保

### 環境記録
- Git commit hash
- Rustコンパイラバージョン
- OS/カーネル情報
- CPU/メモリ情報
- 環境変数設定

### 結果アーカイブ
```
results/
├── 2025-08-27/
│   ├── environment.json
│   ├── raw_data/
│   │   ├── fib_recursive.json
│   │   ├── fib_iterative.json
│   │   └── ...
│   └── summary_report.md
```

## 統計的妥当性

### 測定方法
1. **ウォームアップ**: 最初の10回は除外
2. **繰り返し**: 最低100回実行
3. **外れ値除去**: 上下5%を除外
4. **信頼区間**: 95%信頼区間を算出

### レポート形式
- 平均値 ± 標準偏差
- 中央値（外れ値の影響を排除）
- 最小値・最大値
- ヒストグラム（分布の可視化）

## 論文での提示方法

### 図表案
1. **性能グラフ**: ベンチマーク別の実行時間比較
2. **フォールバック率**: エラー注入量とフォールバック成功率
3. **モジュール性**: 機能追加時の変更行数比較
4. **アーキテクチャ図**: 箱境界とJIT/VMの関係

### 主張の裏付け
- 定量的データ（ベンチマーク結果）
- 定性的分析（コード複雑度、保守性）
- 事例研究（実際の拡張例）