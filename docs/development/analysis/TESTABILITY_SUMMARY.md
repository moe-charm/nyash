# テスト可能性分析サマリー

**分析日**: 2025-10-15
**完全版**: [TESTABILITY_QUALITY_REPORT.md](TESTABILITY_QUALITY_REPORT.md)

---

## 🎯 核心発見（30秒で読める）

### ✅ 優秀な点
- **Result型**: 39ファイル270箇所で一貫したエラーハンドリング
- **Hakorune-VM**: 22テストファイル、100%命令カバレッジ達成
- **防御的**: null チェック475箇所で徹底

### ❌ 最大の問題
- **pipeline.hako**: 504行、116分岐の巨大関数（テスト不可）
- **依存注入ゼロ**: `new MapBox()`直接生成でモック不可
- **統合テスト不足**: VM+Compiler E2Eシナリオ少ない

---

## 🚨 即座に実施すべきアクション（Phase 20.5, Week 1-2）

### アクション #1: pipeline.hako リファクタ（最優先）

**問題**:
```hakorune
// 504行、116分岐の巨大関数（テスト不可）
flow PipelineV2 {
  lower_stage1_to_mir_trace(ast_json, prefer_cfg, trace) {
    // ... 504行の複雑な分岐
  }
}
```

**解決策**: 7つの関数に分割
```hakorune
flow PipelineV2 {
  lower_stage1_to_mir_trace(ast_json, prefer_cfg, trace) {
    local pattern = PatternDetectorBox.detect(ast_json)

    if pattern == "compare" { return me._handle_compare(...) }  // 80行
    if pattern == "call" { return me._handle_call(...) }        // 80行
    if pattern == "method" { return me._handle_method(...) }    // 80行
    // ... 各50-80行
  }
}
```

**工数**: 5日（分離 3日 + テスト 2日）
**効果**: 504行 → 50行×7、循環的複雑度 116 → 50
**ROI**: 🔴 非常に高い（保守性・テスト性が劇的改善）

---

### アクション #2: VM系Boxへの依存注入導入

**問題**:
```hakorune
box MirVmMin {
  run(mir_json) {
    local regs = new MapBox()  // ✗ モック不可
    local mem = new MapBox()   // ✗ 状態観測不可
  }
}
```

**解決策**: コンストラクタ注入
```hakorune
box MirVmMin {
  regs: MapBox
  mem: MapBox

  birth(regs, mem) {
    me.regs = regs
    if me.regs == null { me.regs = new MapBox() }  // 後方互換

    me.mem = mem
    if me.mem == null { me.mem = new MapBox() }
  }

  get_regs() { return me.regs }  // テスト用
  get_mem() { return me.mem }
}
```

**工数**: 3日（改修 2日 + テスト 1日）
**効果**: ユニットテスト可能、状態観測可能
**ROI**: 🟡 高い（テスト網羅率向上）

---

## 📊 品質メトリクス

### 循環的複雑度（Top 5）

| ファイル | 分岐数 | 評価 | アクション |
|---------|--------|------|----------|
| pipeline.hako | 116 | 🔴 危険 | **即座にリファクタ** |
| mir_vm_min.hako | 83 | 🔴 危険 | 分割推奨 |
| mini_vm_binop.hako | 70 | 🟡 注意 | 監視 |
| mir_builder_min.hako | 68 | 🟡 注意 | 監視 |
| stage1_extract_flow.hako | 63 | 🟡 注意 | 監視 |

**基準**: 🟢 0-20（良好）、🟡 21-50（注意）、🔴 51+（危険）

### テストカバレッジ

```
総ファイル: 165 files
テスト: 22 files (13.3%)

カバー率推定:
- Hakorune-VM: 95% （22命令中21命令に専用テスト）
- Compiler: 30% （統合テスト経由）
- 全体: 40%
```

---

## 🗺️ 実装ロードマップ

### Phase 1: 緊急対応（Week 1-2, Phase 20.5）
- [x] **分析完了**: 本レポート作成
- [ ] **pipeline.hako**: 7関数に分割（5日）
- [ ] **MirVmMin**: 依存注入導入（3日）

### Phase 2: 基盤整備（Week 3-6, Phase 20.6）
- [ ] **統合テストフレームワーク**: Parser+Compiler+VM（5日）
- [ ] **Golden Testing**: Rust-VM vs Hako-VM（週2-3）

### Phase 3: 網羅率向上（Week 7-12, Phase 20.7-20.8）
- [ ] **Emit系Box群**: 各1-2日でテスト追加（8日）
- [ ] **Normalizer/Extractor**: テスト追加（4日）
- [ ] **目標**: カバレッジ 40% → 80%

---

## 📚 詳細レポート

完全版（39ページ）: [TESTABILITY_QUALITY_REPORT.md](TESTABILITY_QUALITY_REPORT.md)

**内容**:
1. テストカバレッジ現状（Box別マトリックス）
2. テスト困難箇所の詳細分析（Top 10）
3. テスタビリティ改善提案（コード例付き）
4. エラーハンドリング分析
5. 品質メトリクス（循環的複雑度、関数サイズ）
6. テスト追加優先度マトリックス
7. 実装ロードマップ（Phase別）
8. 付録（テストカバレッジ詳細、参考リソース）

---

**次のステップ**: pipeline.hako リファクタ計画書作成（Task 8）
