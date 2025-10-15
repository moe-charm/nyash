# 統合リファクタリングロードマップ - README

**最終更新**: 2025-10-15
**作成者**: Claude (Task 8 - Integration)

---

## 📖 このドキュメント群について

このディレクトリには、Hakoruneプロジェクト全体の統合リファクタリング計画が含まれています。

### 🎯 目的

- **7,825行（7.9%）の削減** - Legacy code、重複ファイル、未使用コード
- **品質向上** - テスト成功率 91.9% → 95%+、コンパイル警告 4件 → 0件
- **Phase 20.5戦略変更** - 36週間 → 6週間（Hakorune VM完成済み発見により）

---

## 📚 ドキュメント構成

### 1. ⚡ クイックスタート（今すぐ読む）

**[REFACTORING_QUICK_START.md](REFACTORING_QUICK_START.md)**

- **Phase 1を今すぐ実行**（1-2時間で2,383行削減）
- Step-by-stepガイド
- チェックリスト
- トラブルシューティング

**推奨読者**: すべての実装担当者

---

### 2. ⭐ 統合ロードマップ（詳細計画）

**[INTEGRATED_REFACTORING_ROADMAP.md](INTEGRATED_REFACTORING_ROADMAP.md)**

- **Phase 1-3の全体戦略**
- 削減見込み: 7,825行（7.9%）
- 工数見積もり: 11週間
- リスク評価と緩和策
- Phase別タスク詳細

**推奨読者**: アーキテクト、プロジェクトマネージャー、すべての開発者

---

### 3. 📋 リファクタリング索引（ナビゲーション）

**[REFACTORING_INDEX.md](REFACTORING_INDEX.md)**

- すべてのリファクタリング関連ドキュメントへのリンク
- ドキュメント関係図
- Phase別タスク一覧
- 読む順序（推奨）

**推奨読者**: 新規参入者、ドキュメントナビゲーションが必要な人

---

### 4. 📊 分析レポート（背景データ）

**[legacy-code-detection-report.md](legacy-code-detection-report.md)**

- Legacy code特定（調査日: 2025-10-13）
- 削減見積もり: 7,186行（7.2%）
- 優先度別分類（P0/P1/P2）
- Phase 15.6完了待ち項目

**推奨読者**: アーキテクト、レビュアー

---

**[TEST_COMPLEXITY_REPORT.md](TEST_COMPLEXITY_REPORT.md)**

- 185テスト実行（170 PASS / 15 FAIL）
- 失敗テスト根本原因分析
- 複雑度トレンド予測
- Phase 3完了への影響評価

**推奨読者**: QA、テスト担当者

---

## 🚀 今すぐ始める方法

### Step 1: クイックスタートガイドを読む（5分）

```bash
cat docs/development/analysis/REFACTORING_QUICK_START.md
```

### Step 2: Phase 1を実行（1-2時間）

```bash
# バックアップ削除（5分）
rm src/backend/mir_interpreter/handlers/calls/method.rs.bak.1760057047

# BID Codegen削除（30分）
rm -rf src/bid-codegen-from-copilot
rm -rf src/bid-converter-copilot

# Plugin Legacy削除（15分）
rm src/runtime/plugin_box_legacy.rs

# 警告修正（15分）
# type_registry.rs, dispatch.rs, ffi_bridge.rs, mir_json_emit.rs

# テスト（30分）
cargo build --release
tools/smokes/v2/run.sh --profile quick

# コミット（5分）
git add -A
git commit -m "refactor(phase1): Quick Wins完了 - 2,383行削減"
git push
```

### Step 3: 統合ロードマップを読む（30分）

```bash
cat docs/development/analysis/INTEGRATED_REFACTORING_ROADMAP.md
```

---

## 📊 Phase別概要

### Phase 1: Quick Wins（1-2時間）⚡ 最優先

- **削減**: 2,383行（2.4%）
- **リスク**: 極低（参照ゼロファイル）
- **前提条件**: なし（今すぐ実行可）

**タスク**:
1. バックアップファイル削除: 327行
2. BID Codegen実験コード削除: 1,894行
3. Plugin Legacy Proxy削除: 158行
4. 未使用警告修正: 4行

---

### Phase 2: 構造改善（4週間）🏗️ 高優先

- **削減**: 4,197行（4.2%）
- **リスク**: 中（Plugin安定化待ち）
- **前提条件**: Phase 20.5完了（Hakorune VM検証・統合）

**Phase 20.5（6週間）**:
- Week 1-2: VM検証・テスト拡充
- Week 3-4: Golden Testing（Rust-VM vs Hako-VM）
- Week 5: CLI統合（--backend vm-hako）
- Week 6: ドキュメント整備

**Legacy削除**:
- Legacy VM handlers削除: 1,145行
- src/boxes/削除: 3,000行
- MIR Builder legacy削除: 52行

---

### Phase 3: 長期最適化（6週間）🔧 中優先

- **削減**: 1,245行（1.2%）
- **リスク**: 中-高（設計変更伴う）
- **前提条件**: Phase 2完了

**Selfhost Compiler**:
- 重複ファイル統一（14組）: 300行
- parser_box分割（921行→3箱）
- pipeline_v2構造化

**Backend統合**:
- Cranelift JIT削除判断: 45行
- AOT Backend統合判断: 350行
- LLVM Legacy削除: 500行

**品質向上**:
- INTERFACES.md v2.0
- TODO/FIXME整理: 50行

---

## 🎯 成果指標（KPI）

### 削減目標

| Phase | 削減行数 | 削減率 | 所要時間 |
|-------|---------|--------|---------|
| Phase 1 | 2,383 | 2.4% | 1-2時間 ⚡ |
| Phase 2 | 4,197 | 4.2% | 4週間 |
| Phase 3 | 1,245 | 1.2% | 6週間 |
| **合計** | **7,825** | **7.9%** | **11週間** |

### 品質指標

| 指標 | 現状 | Phase 1後 | Phase 2後 | Phase 3後 |
|-----|------|-----------|-----------|-----------|
| Rust総行数 | 99,439 | 97,056 | 92,859 | 91,614 |
| テスト成功率 | 91.9% | 91.9% | 95%+ | 95%+ |
| コンパイル警告 | 4件 | **0件** | **0件** | **0件** |
| Legacy残存 | Yes | Yes | **No** | **No** |
| src/boxes/ | 57ファイル | 57 | **0** | **0** |

---

## 🔍 Critical Discovery（2025-10-14）

### 🎉 Hakorune VMは100%完成済み！

**旧計画**: 36週間（VM実装 8週間 + Dispatch 6週間 + ...）
**新計画**: 6週間（検証 2週間 + Golden Test 2週間 + CLI統合 2週間）

**発見内容**:
- ✅ 3,413行のHakoruneコード
- ✅ 22ハンドラー（16 MIR命令 + 6拡張 = 138%カバレッジ）
- ✅ 26+の包括的テスト
- ✅ @match-based dispatch
- ✅ Result-based error handling
- ✅ 実装期間: わずか8日間（2025-10-05 → 10-13）

**影響**: Phase 20.5は「実装」から「検証・統合」に戦略変更

**詳細**: [Phase 20.5 README](../roadmap/phases/phase-20.5/README.md) | [Hakorune VM Discovery](../roadmap/phases/phase-20.5/HAKORUNE_VM_DISCOVERY.md)

---

## 🚨 リスク評価

### リスク管理の基本原則

1. **段階的実行**: Phase 1 → Phase 2 → Phase 3（各Phase完了判定あり）
2. **Fail-Fast**: エラーは隠さず即座に失敗
3. **Git履歴保持**: 削除前に必ずcommit（復元可能）
4. **ロールバック可能**: いつでも前の状態に戻せる

### 主要リスク

| リスク | 確率 | 影響 | 軽減策 |
|--------|------|------|--------|
| Phase 20.5統合失敗 | 低 | 高 | Golden Testing徹底、段階的統合 |
| Plugin安定性不足 | 中 | 高 | 1週間連続テスト、Legacy一時保持 |
| Selfhost分割ミス | 中 | 中 | 責務分析慎重実施、INTERFACES.md同期 |
| Backend統合判断ミス | 中 | 中-高 | User判断を仰ぐ、docs/proposals/ideas/へ移動 |
| 工数超過（Phase 3） | 高 | 低 | Phase毎に完了判定、80/20ルール |

---

## 💡 よくある質問（FAQ）

### Q1: どのPhaseから始めればいいですか？

**A**: **Phase 1を今すぐ実行**してください。Phase 2/3はPhase 1完了後に判断できます。

---

### Q2: Phase 1だけ実行して、Phase 2/3をスキップしてもいいですか？

**A**: はい、問題ありません。Phase 1は独立しており、即座に効果があります。Phase 2/3は優先度に応じて判断してください。

---

### Q3: Phase 20.5はいつ始まりますか？

**A**: 2025-12-21開始予定（Phase 15.77完了後）。ただし、Hakorune VM発見により計画変更の可能性があります。

---

### Q4: テスト失敗が出た場合はどうすればいいですか？

**A**:
1. 詳細ログ確認: `NYASH_CLI_VERBOSE=1 tools/smokes/v2/run.sh --profile quick`
2. 最後の正常なコミットに戻る: `git revert <commit-hash>`
3. [トラブルシューティングガイド](REFACTORING_QUICK_START.md)参照

---

### Q5: Backend統合判断はどうすればいいですか？

**A**: User（tomoakiさん）判断を仰いでください。Cranelift/AOT/LLVM Legacyの将来計画を確認してから決定します。

---

## 📞 サポート・相談

### 質問先

- **統合ロードマップ**: Claude
- **Phase 20.5**: tomoaki + Claude
- **Selfhost整理**: Claude + User
- **Backend統合**: User判断必須
- **Plugin安定性**: ChatGPT + User

### ドキュメント不明点

- [REFACTORING_INDEX.md](REFACTORING_INDEX.md) でドキュメント一覧確認
- 各Phase詳細は [INTEGRATED_REFACTORING_ROADMAP.md](INTEGRATED_REFACTORING_ROADMAP.md) 参照

---

## 🎯 次のアクション

### 今日実行すべきこと

```bash
# 1. クイックスタートガイドを読む（5分）
cat docs/development/analysis/REFACTORING_QUICK_START.md

# 2. Phase 1を実行（1-2時間）
# → バックアップ削除、BID Codegen削除、Plugin Legacy削除、警告修正

# 3. テスト＆コミット
cargo build --release
tools/smokes/v2/run.sh --profile quick
git add -A && git commit -m "refactor(phase1): Quick Wins完了" && git push
```

### Phase 2準備（Phase 20.5進捗確認）

```bash
# Phase 20.5進捗確認
cat docs/development/roadmap/phases/phase-20.5/README.md

# Hakorune VMテスト実行
cd selfhost/hakorune-vm
for test in tests/*.hako; do
    NYASH_DISABLE_PLUGINS=1 ../../target/release/hako "$test"
done
```

---

## 🎊 最終メッセージ

### 核心原則

1. **段階的実行**: Phase 1 → Phase 2 → Phase 3
2. **Fail-Fast**: エラーは隠さず即座に失敗
3. **Box-First**: すべてを箱で分離・固定
4. **80/20ルール**: 完璧より進捗

### 今日から始められること

✅ **Phase 1を今すぐ実行**（1-2時間で2,383行削減）

```bash
# クイックスタートガイドを開く
cat docs/development/analysis/REFACTORING_QUICK_START.md

# Step 1-6を順次実行
# → 2,383行削減、コンパイル警告0件
```

---

**🚀 Let's refactor with Box-First philosophy and Fail-Fast culture!**

**📅 作成日**: 2025-10-15
**👤 作成者**: Claude (Task 8 - Integration)
**📊 分析基盤**: Task 1-7結果 + Phase 20.5 Discovery + Master Roadmap
**🎯 目標**: 実行可能・段階的・後戻り可能なリファクタリング計画

---

## 📚 関連リソース

### このディレクトリ内

- [INTEGRATED_REFACTORING_ROADMAP.md](INTEGRATED_REFACTORING_ROADMAP.md) - 統合ロードマップ ⭐
- [REFACTORING_QUICK_START.md](REFACTORING_QUICK_START.md) - クイックスタート ⚡
- [REFACTORING_INDEX.md](REFACTORING_INDEX.md) - ドキュメント索引
- [legacy-code-detection-report.md](legacy-code-detection-report.md) - Legacy検出
- [TEST_COMPLEXITY_REPORT.md](TEST_COMPLEXITY_REPORT.md) - テスト分析

### Phase 20.5関連

- [Phase 20.5 README](../roadmap/phases/phase-20.5/README.md) ⭐
- [Hakorune VM Discovery](../roadmap/phases/phase-20.5/HAKORUNE_VM_DISCOVERY.md) ⭐
- [Strategy Reconciliation](../roadmap/phases/phase-20.5/STRATEGY_RECONCILIATION.md)

### Selfhost Compiler関連

- [Selfhost Super Refactoring Master Plan](../proposals/ideas/refactoring/selfhost-super-refactoring/reports/refactoring_master_plan.md)

### 開発方針

- [CLAUDE.md](../../../CLAUDE.md) - 箱理論・Fail-Fast原則
- [00_MASTER_ROADMAP.md](../roadmap/phases/00_MASTER_ROADMAP.md) - 全体計画
