# リファクタリング関連ドキュメント索引

**最終更新**: 2025-10-15

---

## 🎯 メインドキュメント（必読）

### 1. 統合リファクタリングロードマップ ⭐最優先
**ファイル**: [INTEGRATED_REFACTORING_ROADMAP.md](INTEGRATED_REFACTORING_ROADMAP.md)

**内容**:
- 3フェーズ戦略（Phase 1-3）
- 削減見込み: 7,825行（7.9%）
- 工数見積もり: 11週間
- リスク評価と緩和策

**対象読者**: すべての開発者

---

### 2. クイックスタートガイド ⚡今すぐ実行
**ファイル**: [REFACTORING_QUICK_START.md](REFACTORING_QUICK_START.md)

**内容**:
- Phase 1（今すぐ実行可能）の手順
- 30分で2,383行削減
- チェックリスト
- トラブルシューティング

**対象読者**: 実装担当者

---

## 📊 分析レポート

### 3. Legacy Code検出レポート
**ファイル**: [legacy-code-detection-report.md](legacy-code-detection-report.md)

**内容**:
- レガシーコード特定（調査日: 2025-10-13）
- 削減見積もり: 7,186行（7.2%）
- 優先度別分類（P0/P1/P2）
- Phase 15.6完了待ち項目

**対象読者**: アーキテクト、レビュアー

---

### 4. テスト複雑度レポート
**ファイル**: [TEST_COMPLEXITY_REPORT.md](TEST_COMPLEXITY_REPORT.md)

**内容**:
- 185テスト実行（170 PASS / 15 FAIL）
- 失敗テスト根本原因分析
- 複雑度トレンド予測
- Phase 3完了への影響評価

**対象読者**: QA、テスト担当者

---

## 🏗️ Selfhost Compiler関連

### 5. Selfhost Super Refactoring Master Plan
**ファイル**: [../proposals/ideas/refactoring/selfhost-super-refactoring/reports/refactoring_master_plan.md](../proposals/ideas/refactoring/selfhost-super-refactoring/reports/refactoring_master_plan.md)

**内容**:
- Selfhost compiler詳細計画
- 重複ファイル14組の統一
- parser_box.hako分割（921行 → 3箱）
- pipeline_v2/構造整理

**対象読者**: Selfhost compiler開発者

---

### 6. Selfhost Compiler関連分析（複数）
**ディレクトリ**: [../proposals/ideas/refactoring/selfhost-super-refactoring/reports/](../proposals/ideas/refactoring/selfhost-super-refactoring/reports/)

**ファイル一覧**:
- `refactoring_executive_summary.md` - エグゼクティブサマリー
- `refactoring_risk_matrix.md` - リスクマトリックス
- `refactoring_gantt_chart.md` - ガントチャート
- `selfhost_compiler_structure.md` - コンパイラ構造分析

**対象読者**: プロジェクトマネージャー

---

## 🎯 Phase別ドキュメント

### Phase 20.5（Hakorune VM検証・統合）

#### 7. Phase 20.5 README ⭐重大更新
**ファイル**: [../roadmap/phases/phase-20.5/README.md](../roadmap/phases/phase-20.5/README.md)

**内容**:
- Critical Discovery（2025-10-14）: Hakorune VM完成済み
- 36週間 → 6週間に短縮
- 週次計画（Week 1-6）
- Golden Testing戦略

**対象読者**: すべての開発者

---

#### 8. Hakorune VM Discovery Report ⭐発見レポート
**ファイル**: [../roadmap/phases/phase-20.5/HAKORUNE_VM_DISCOVERY.md](../roadmap/phases/phase-20.5/HAKORUNE_VM_DISCOVERY.md)

**内容**:
- 22ハンドラー完全実装確認
- 3,413行、8日間で実装
- アーキテクチャ詳解
- 検証戦略

**対象読者**: アーキテクト、VM開発者

---

#### 9. Strategy Reconciliation
**ファイル**: [../roadmap/phases/phase-20.5/STRATEGY_RECONCILIATION.md](../roadmap/phases/phase-20.5/STRATEGY_RECONCILIATION.md)

**内容**:
- 戦略変更の理由
- 旧計画 vs 新計画
- Option A（Pure Hakorune）vs Option B（HostBridge）

**対象読者**: アーキテクト、意思決定者

---

## 📚 その他の重要リソース

### 10. Master Roadmap
**ファイル**: [../roadmap/phases/00_MASTER_ROADMAP.md](../roadmap/phases/00_MASTER_ROADMAP.md)

**内容**:
- 全Phase概要（Phase 1-30+）
- 現在位置: Phase 19（@enum/@match）
- Phase 15系完了状況（85-90%）

**対象読者**: すべての開発者

---

### 11. CLAUDE.md（開発方針）
**ファイル**: [../../../CLAUDE.md](../../../CLAUDE.md)

**内容**:
- 箱理論（Box-First）
- Fail-Fast原則
- 80/20ルール
- AI協調開発方針

**対象読者**: すべての開発者

---

## 🗺️ ドキュメント関係図

```
統合リファクタリングロードマップ（本索引）
├── Phase 1: Quick Wins
│   ├── クイックスタートガイド ⚡
│   └── Legacy Code検出レポート
├── Phase 2: 構造改善
│   ├── Phase 20.5 README ⭐
│   ├── Hakorune VM Discovery ⭐
│   ├── Strategy Reconciliation
│   └── Legacy Code検出レポート
└── Phase 3: 長期最適化
    ├── Selfhost Super Refactoring Master Plan
    ├── Selfhost Compiler関連分析
    └── テスト複雑度レポート

補助ドキュメント:
├── Master Roadmap（全体計画）
└── CLAUDE.md（開発方針）
```

---

## 📋 読む順序（推奨）

### 新規参入者向け
1. **CLAUDE.md** - 開発方針理解
2. **Master Roadmap** - 全体像把握
3. **統合リファクタリングロードマップ** - リファクタリング計画
4. **クイックスタートガイド** - 実装開始

### 実装担当者向け
1. **クイックスタートガイド** ⚡ - 今すぐ実行
2. **統合リファクタリングロードマップ** - 詳細計画
3. **Legacy Code検出レポート** - 削除対象確認
4. **Phase 20.5 README** - VM統合計画

### アーキテクト向け
1. **統合リファクタリングロードマップ** - 全体戦略
2. **Hakorune VM Discovery** - 重大発見
3. **Strategy Reconciliation** - 戦略変更理由
4. **Selfhost Super Refactoring Master Plan** - Selfhost詳細

### QA/テスト担当者向け
1. **テスト複雑度レポート** - 現状分析
2. **統合リファクタリングロードマップ** - テスト戦略
3. **Phase 20.5 README** - Golden Testing計画

---

## 🎯 Phase別タスク一覧

### Phase 1: Quick Wins（今すぐ実行可）

**参照**: [クイックスタートガイド](REFACTORING_QUICK_START.md)

| タスク | 削減 | 工数 | 担当 |
|-------|------|------|------|
| バックアップ削除 | 327行 | 5分 | User/Claude |
| BID Codegen削除 | 1,894行 | 30分 | User/Claude |
| Plugin Legacy削除 | 158行 | 15分 | Claude |
| 警告修正 | 4行 | 15分 | Claude |
| **合計** | **2,383行** | **1-2時間** | - |

---

### Phase 2: 構造改善（4週間、Phase 20.5後）

**参照**: [統合リファクタリングロードマップ](INTEGRATED_REFACTORING_ROADMAP.md) Phase 2

| タスク | 削減 | 工数 | 担当 |
|-------|------|------|------|
| Phase 20.5（VM検証） | - | 3週間 | tomoaki+Claude |
| Legacy handlers削除 | 1,145行 | 3日 | ChatGPT+User |
| src/boxes/削除 | 3,000行 | 1-2週間 | ChatGPT+User |
| MIR Builder legacy削除 | 52行 | 3時間 | Claude |
| **合計** | **4,197行** | **4週間** | - |

---

### Phase 3: 長期最適化（6週間、Phase 2後）

**参照**: [統合リファクタリングロードマップ](INTEGRATED_REFACTORING_ROADMAP.md) Phase 3

| タスク | 削減 | 工数 | 担当 |
|-------|------|------|------|
| Selfhost重複統一 | 300行 | 2-3日 | Claude+User |
| parser_box分割 | 0行 | 3時間 | Claude |
| pipeline_v2構造化 | 0行 | 1-2時間 | Claude |
| Backend統合判断 | 895行 | 1週間 | User+ChatGPT |
| TODO/FIXME整理 | 50行 | 3時間 | Claude |
| **合計** | **1,245行** | **6週間** | - |

---

## 📊 統計サマリー

### 削減見込み合計
- **Phase 1**: 2,383行（2.4%）- 今すぐ実行可
- **Phase 2**: 4,197行（4.2%）- Phase 20.5後
- **Phase 3**: 1,245行（1.2%）- Phase 2後
- **合計**: **7,825行（7.9%）**

### コードベース現状
- **Rust**: 99,439行（762ファイル）
- **Hakorune**: 13,417行（165ファイル）
- **テスト成功率**: 170/185 (91.9%)
- **コンパイル警告**: 4件 → 0件（Phase 1後）

### 工数見積もり
- **Phase 1**: 1-2時間（今すぐ）
- **Phase 2**: 4週間（Phase 20.5含む）
- **Phase 3**: 6週間
- **合計**: **11週間**

---

## 🚨 重要な注意事項

### 実行順序厳守
1. ✅ **Phase 1を今すぐ実行可** - 他Phase待ち不要
2. ⏳ **Phase 2はPhase 20.5完了待ち** - Hakorune VM統合が前提
3. ⏳ **Phase 3はPhase 2完了待ち** - Legacy削除が前提

### リスク管理
- **段階的実行**: 各Phase完了時にテスト実行
- **Git履歴保持**: 削除前に必ずcommit
- **Fail-Fast**: エラーは隠さず即座に報告
- **ロールバック可能**: いつでも前の状態に戻せる

---

## 💡 次のアクション

### 今日実行すべきこと（Phase 1）

```bash
# 1. クイックスタートガイドを開く
cat docs/development/analysis/REFACTORING_QUICK_START.md

# 2. Step 1-6を順次実行（1-2時間）
# → 2,383行削減

# 3. コミット＆プッシュ
git add -A
git commit -m "refactor(phase1): Quick Wins完了"
git push
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

## 📞 質問・相談先

### ドキュメント不明点
- **統合ロードマップ**: Claude
- **Phase 20.5**: tomoaki + Claude
- **Selfhost整理**: Claude + User

### 実装判断
- **Backend統合**: User判断必須
- **Plugin安定性**: ChatGPT + User
- **リスク評価**: Claude

---

**作成日**: 2025-10-15
**作成者**: Claude (Task 8 - Integration)
**目的**: リファクタリング関連ドキュメントの一元管理・ナビゲーション
