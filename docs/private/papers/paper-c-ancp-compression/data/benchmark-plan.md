# ANCP Benchmark Plan - 論文用データ収集

## 📊 実験設計

### 1. 圧縮性能ベンチマーク

#### データセット
```
datasets/
├── small/           # 100-1000 LOC サンプル
├── medium/          # 1000-10000 LOC モジュール  
├── large/           # 10000+ LOC アプリケーション
└── nyash-compiler/  # 80k LOC 自己ホスティングコンパイラ
```

#### 測定指標
| Metric | Unit | Purpose |
|--------|------|---------|
| Character Reduction | % | ファイルサイズ削減 |
| Token Reduction | % | AI理解性向上 |
| AST Node Count | count | 構造複雑度 |
| Compression Time | ms | 実用性評価 |
| Decompression Time | ms | 開発体験 |

### 2. 可逆性検証

#### ラウンドトリップテスト
```rust
#[test]
fn test_reversibility() {
    for sample in test_samples() {
        let compressed = ancp.compress(sample, Level::Fusion);
        let restored = ancp.decompress(compressed);
        assert_eq!(normalize(sample), normalize(restored));
        
        // MIR等価性も検証
        let mir_original = compile_to_mir(sample);
        let mir_restored = compile_to_mir(restored);
        assert_eq!(mir_original, mir_restored);
    }
}
```

#### 測定データ
- **サンプル数**: 10,000ファイル
- **成功率**: 100%（目標）
- **エラー分析**: 失敗ケースの詳細分析

### 3. AI効率性評価

#### LLM Token Consumption
| Model | Context | Original | ANCP | Improvement |
|-------|---------|----------|------|-------------|
| GPT-4 | 128k | 20k LOC | 40k LOC | 2.0x |
| Claude | 200k | 40k LOC | 80k LOC | 2.0x |
| Gemini | 100k | 20k LOC | 40k LOC | 2.0x |

#### Code Understanding Tasks
```python
# AI理解性評価スクリプト
def evaluate_ai_understanding(model, code_samples):
    results = []
    
    for original, ancp in code_samples:
        # 元のコードでのタスク
        original_score = model.complete_code_task(original)
        
        # ANCPでのタスク
        ancp_score = model.complete_code_task(ancp)
        
        results.append({
            'original_score': original_score,
            'ancp_score': ancp_score,
            'compression_ratio': calculate_compression(original, ancp)
        })
    
    return analyze_correlation(results)
```

### 4. 実用性評価

#### 開発ワークフロー
```bash
# 通常の開発フロー
edit file.nyash          # P層で開発
nyashc --compact file.c  # C層で配布
nyashc --fusion file.f   # F層でAI投入
```

#### 測定項目
- 開発効率（P層での作業時間）
- 変換速度（P→C→F変換時間）  
- デバッグ効率（エラーの逆引き精度）

---

## 📈 予想される結果

### 圧縮率
- **Layer C**: 48% ± 5% (Standard deviation)
- **Layer F**: 90% ± 3% (Consistently high)
- **Comparison**: 1.6x better than Terser

### 可逆性
- **Success Rate**: 99.9%+ (目標)
- **Edge Cases**: 特殊文字・Unicode・コメント処理

### AI効率
- **Context Expansion**: 2-3x capacity increase
- **Understanding Quality**: No degradation (hypothesis)

---

## 🔧 実験プロトコル

### Phase 1: 基本機能実装
1. P→C→F変換器
2. ソースマップ生成器  
3. 可逆性テストスイート

### Phase 2: 大規模評価
1. 10,000サンプルでの自動評価
2. 各種メトリクス収集
3. エラーケース分析

### Phase 3: AI評価
1. 3つの主要LLMでの効率測定
2. コード理解タスクでの性能比較
3. 実用的な開発シナリオでのテスト

### Phase 4: 論文執筆
1. 結果の統計解析
2. 関連研究との詳細比較
3. 査読対応の準備

---

## 📝 データ収集チェックリスト

- [ ] **Compression Benchmarks**: 各レイヤーでの削減率
- [ ] **Reversibility Tests**: 10k samples roundtrip verification
- [ ] **AI Efficiency**: LLM token consumption measurement  
- [ ] **Performance**: Transformation speed benchmarks
- [ ] **Real-world**: Self-hosting compiler case study
- [ ] **User Study**: Developer experience evaluation
- [ ] **Comparison**: Head-to-head with existing tools

---

## 🎯 論文の説得力

### 定量的証拠
- 圧縮率の客観的測定
- 可逆性の数学的証明
- AI効率の実証データ

### 実用的価値  
- 動作するプロトタイプ
- 実際のコンパイラでの検証
- 開発ツール統合

### 学術的新規性
- 90%可逆圧縮の達成
- AI最適化の新パラダイム
- Box-First設計の有効性実証

---

**次のステップ**: データ収集の自動化スクリプト実装