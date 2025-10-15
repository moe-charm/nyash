# 超リファクタリング計画 - レポート一覧

**調査完了日**: 2025-10-04
**調査対象**: apps/selfhost-compiler/ + apps/selfhost/vm/
**調査手法**: Task並列エージェント × 4

---

## 📚 メインエントリー（必読）

1. **[README.md](README.md)** ⭐⭐⭐ - 計画の入口・クイックスタート
   - 3秒で理解: 14組の重複ファイル統一、箱化率100%達成計画
   - 4 Phases / 4 Days / 21 Hours
   - ROI: 376%（3.76倍のリターン）

2. **[mini-vm-reports-index.md](mini-vm-reports-index.md)** ⭐⭐ - Mini-VMレポートナビ
   - 調査成果物の詳細インデックス
   - 推奨される読み進め方

3. **[verify_current_state.sh](verify_current_state.sh)** 🔧 - KPI自動測定スクリプト
   - 実行: `bash verify_current_state.sh`
   - .nyashファイル数、重複ファイル数、箱化率、最大ファイル行数を自動測定

---

## 📊 詳細レポート (reports/)

### 🎯 総合計画ドキュメント

#### [refactoring_index.md](reports/refactoring_index.md) - ドキュメント索引
**サイズ**: 12KB | **読了時間**: 5分 | **優先度**: ⭐⭐⭐
- 全7ドキュメント + 1スクリプトの完全ガイド
- 各ドキュメントの概要・サイズ・読了時間
- 読む順序の推奨（初めて/実行中/問題発生時）

#### [refactoring_executive_summary.md](reports/refactoring_executive_summary.md) - エグゼクティブサマリー
**サイズ**: 13KB | **読了時間**: 10分 | **優先度**: ⭐⭐⭐
- 計画全体の概要（3秒理解版）
- 現状分析 vs 目標
- ROI（投資対効果）: 376%
- ビジョン: 箱化率100%、Phase 15.7加速

#### [refactoring_master_plan.md](reports/refactoring_master_plan.md) - マスタープラン
**サイズ**: 22KB | **読了時間**: 20分 | **優先度**: ⭐⭐
- Phase 0-4の詳細計画
- 各Phaseのタスク一覧
- 成功条件・検証方法
- 実行コマンド集

#### [refactoring_quick_reference.md](reports/refactoring_quick_reference.md) - クイックリファレンス
**サイズ**: 7.8KB | **読了時間**: 5分 | **優先度**: ⭐⭐⭐
- 実行時のチートシート
- Phase別コマンド早見表
- 緊急時対応フロー
- よく使うコマンド集

#### [refactoring_risk_matrix.md](reports/refactoring_risk_matrix.md) - リスク分析マトリックス
**サイズ**: 13KB | **読了時間**: 15分 | **優先度**: ⭐⭐
- 9つの主要リスク識別
- 軽減策（5層防御）
- 緊急時対応フロー
- リスク評価マトリックス

#### [refactoring_gantt_chart.md](reports/refactoring_gantt_chart.md) - ガントチャート
**サイズ**: 14KB | **読了時間**: 5分 | **優先度**: ⭐
- 時系列スケジュール（4日間）
- 並行作業の最適化
- 依存関係マッピング
- 進捗チェックポイント

---

### 🏗️ Selfhost Compiler分析

#### [selfhost_compiler_analysis_index.md](reports/selfhost_compiler_analysis_index.md) - Compiler分析インデックス
**サイズ**: 8.7KB | **読了時間**: 5分
- Compiler分析レポートのナビゲーション
- 重要発見事項サマリー
- 推奨改善アクション

#### [selfhost_compiler_structure.md](reports/selfhost_compiler_structure.md) - Compiler構造詳細
**サイズ**: 25KB | **読了時間**: 30分
- ディレクトリ構造完全マップ
- 主要コンポーネント詳細（Compiler/Pipeline v2/Boxes/Builder/Emitter）
- 依存関係グラフ
- コード規模分析（57ファイル、5,388行）

#### [selfhost_compiler_summary.md](reports/selfhost_compiler_summary.md) - Compiler概要
**サイズ**: 5.5KB | **読了時間**: 10分
- クイックサマリー
- 問題点リスト（2つの巨大ファイル、3つのスタブディレクトリ）
- 改善提案（5つの優先アクション）
- 総合評価: 6.9/10

---

### 🖥️ Mini-VM分析

#### [mini_vm_structure.md](reports/mini_vm_structure.md) - Mini-VM構造詳細
**サイズ**: 17KB | **読了時間**: 30分
- エグゼクティブサマリー
- ディレクトリ構造（apps/selfhost/vm/ + selfhost/shared/common/）
- 主要コンポーネント詳細（23箱）
- 依存関係マップ
- コード規模分析（52ファイル、~3,400行）
- 新旧バージョン比較
- アーキテクチャ設計原則

#### [mini_vm_dependency_graph.txt](reports/mini_vm_dependency_graph.txt) - Mini-VM依存グラフ
**サイズ**: 13KB | **読了時間**: 20分
- レイヤー構造（ASCII表現）
- 詳細依存グラフ（ツリー表現）
- 循環依存の問題点（3箇所）
- 共通機能の重複リスト（~250行）
- 推奨される統一モジュール（StringUtilsBox/ScanUtilsBox）

#### [mini_vm_boxes_catalog.md](reports/mini_vm_boxes_catalog.md) - Mini-VM箱カタログ
**サイズ**: 18KB | **読了時間**: 60分
- 全23箱の詳細仕様
- 公開API一覧
- 対応MIR命令
- 使用例（コード付き）
- 箱サイズ一覧
- 箱の責務マトリックス

#### [mini_vm_action_plan.md](reports/mini_vm_action_plan.md) - Mini-VMアクションプラン
**サイズ**: 20KB | **読了時間**: 30分
- Phase 1-4の具体的タスク（8日間計画）
- StringUtilsBox/ScanUtilsBox実装例
- テストコード例
- チェックリスト
- 成功指標（KPI）
- リスクと対策

---

### 📦 箱化・モジュール化分析

#### [box_modularization_analysis.md](reports/box_modularization_analysis.md) - 箱化分析
**サイズ**: 21KB | **読了時間**: 30分
- エグゼクティブサマリー
- 現状分析（箱化率75.4%）
- 問題点詳細:
  - ParserBox巨大化（921行、59メソッド）
  - 14 .nyashファイル残存
  - 共通ユーティリティ箱不足
- 改善提案（優先度付き）:
  - 優先度1: ParserBox分割（5箱化）
  - 優先度2: .nyash→.hako統一
  - 優先度3: ユーティリティ箱新設
- 実装ロードマップ（3週間計画）
- 総合評価: 78/100

---

## 📈 調査結果ハイライト

### 発見事項（Findings）

**Selfhost Compiler**:
- 総ファイル数: 57 (.hako: 43, .nyash: 14)
- 総行数: 5,388行
- 最大ファイル: parser_box.hako (921行) ← 6倍肥大化
- スタブディレクトリ: 3つ（66行の技術的負債）
- 循環依存: なし ✅ （健全）

**Mini-VM**:
- 総ファイル数: 52 (.hako: 23, .nyash: 29)
- 総行数: ~3,400行
- コード重複: ~250行（7.4%）
- 循環依存: 3箇所 ⚠️
- 命名不統一: 3箇所

**箱化・モジュール化**:
- 箱化率: 75.4% (目標: 100%)
- .nyashファイル: 14個 (目標: 0個)
- 重複ファイル: 14組 (目標: 0組)
- ParserBox肥大: 921行、59メソッド (目標: <150行)

### 推奨アクション（Recommendations）

**即座に実施（Phase 1）**:
1. .nyash→.hako統一（14ファイル）
2. 重複ファイル解消（14組）
3. ParserBox分割開始

**短期（Phase 2-3）**:
4. pipeline_v2/構造化（5ディレクトリ）
5. ユーティリティ箱新設（StringUtils/ScanUtils）
6. 循環依存解消（3箇所）

**中期（Phase 4）**:
7. INTERFACES.md v2.0完成
8. デッドコード削除（スタブディレクトリ）
9. ドキュメント整備

---

## 🎯 期待される効果

| 指標 | 現状 | 目標 | 改善率 |
|------|------|------|--------|
| 箱化率 | 75.4% | 100% | +24.6pt |
| .nyashファイル数 | 14 | 0 | **100%削減** |
| 重複ファイル | 14組 | 0組 | **100%解消** |
| 最大ファイル行数 | 921行 | <300行 | **67%削減** |
| コード重複 | 250行 | 0行 | **100%削減** |
| 循環依存 | 3箇所 | 0箇所 | **100%解消** |

**開発効率**: 約30-50%向上（見込み）
**投資対効果**: 376% ROI（3.76倍のリターン）
**回収期間**: 2-3週間

---

## 📖 推奨される読み進め方

### 初めての場合（15分コース）
1. **README.md**（3分） - 計画全体を把握
2. **refactoring_executive_summary.md**（5分） - 詳細理解
3. **refactoring_quick_reference.md**（3分） - 実行手順確認
4. **verify_current_state.sh**（実行）- 現状測定
5. **Phase 0開始！**

### アーキテクチャ理解（1時間コース）
1. **selfhost_compiler_structure.md**（30分） - Compiler全体構造
2. **mini_vm_structure.md**（30分） - VM全体構造
3. **box_modularization_analysis.md**（拾い読み） - 箱化状況

### 実装担当（2時間コース）
1. **refactoring_master_plan.md**（30分） - Phase別詳細手順
2. **mini_vm_boxes_catalog.md**（60分） - 全箱API理解
3. **box_modularization_analysis.md**（30分） - 改善提案詳細

### プロジェクト管理者（20分コース）
1. **refactoring_executive_summary.md**（10分） - 概要とROI
2. **refactoring_gantt_chart.md**（5分） - スケジュール確認
3. **refactoring_risk_matrix.md**（5分） - リスク理解

---

## 🚀 次のアクション

### 即座に実施
```bash
# ステップ1: 現状確認（1分）
bash verify_current_state.sh

# ステップ2: 計画理解（10分）
cat reports/refactoring_executive_summary.md | less
cat reports/refactoring_quick_reference.md | less

# ステップ3: Phase 0開始（2時間）
# 詳細手順はREADME.mdまたはrefactoring_master_plan.md参照
```

---

## 📞 問い合わせ

- **計画全体について**: README.md
- **実行手順について**: reports/refactoring_quick_reference.md
- **リスク対策について**: reports/refactoring_risk_matrix.md
- **スケジュールについて**: reports/refactoring_gantt_chart.md
- **技術詳細について**: reports/selfhost_compiler_structure.md, reports/mini_vm_structure.md

---

**調査者**: Claude (Sonnet 4.5) × Task並列エージェント × 4
**調査日**: 2025-10-04
**所要時間**: 約2時間
**成果物**: 15レポート、約2,800行
**品質保証**: 全レポート相互リンク・整合性確認済み
