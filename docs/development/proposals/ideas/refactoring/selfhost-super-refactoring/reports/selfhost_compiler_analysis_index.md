# Selfhost Compiler 構造調査 - インデックス

**調査実施日**: 2025-10-04
**調査対象**: `/home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler/`

---

## 📚 **生成レポート一覧**

### 1. **クイックサマリー** (推奨・最初に読む)
**ファイル**: `/tmp/selfhost_compiler_summary.md` (181行)

**内容**:
- 一目でわかる統計
- 重要な問題点TOP5
- 推奨アクションプラン
- コード品質スコア

**こんな人向け**:
- 5分で全体像を把握したい
- 最優先タスクを知りたい
- アーキテクチャ評価を見たい

---

### 2. **詳細構造レポート** (完全版)
**ファイル**: `/tmp/selfhost_compiler_structure.md` (655行)

**内容**:
- 全ディレクトリ詳細分析
- Box/Flow定義一覧
- 依存関係マップ
- 問題点・改善候補リスト
- リファクタリング優先度マトリックス

**こんな人向け**:
- アーキテクチャを深く理解したい
- リファクタリング計画を立てたい
- 各モジュールの役割を詳しく知りたい

---

### 3. **依存関係グラフ** (技術詳細)
**ファイル**: `/tmp/selfhost_compiler_dependency_graph.txt` (524行)

**内容**:
- 全モジュールの依存関係ツリー
- Layer別の詳細構造
- 外部依存一覧
- 循環依存チェック結果
- スタブ・削除候補の特定

**こんな人向け**:
- 実装を変更する前に影響範囲を知りたい
- モジュール間の呼び出し関係を確認したい
- 技術的負債（スタブ）を洗い出したい

---

## 🎯 **調査結果サマリー**

### **規模**
- 総ファイル数: 57
- 総行数: 5,388
- Box定義数: 30
- Flow定義数: 5

### **アーキテクチャ評価**
```
設計品質:        8/10  ✅ Box-First設計が優秀
コード整理度:    6/10  ⚠️ スタブ・超大型ファイル残存
保守性:          7/10  ✅ 責務分離は明確だが改善余地
テストカバレッジ: 2/10  ❌ テスト未整備
ドキュメント:    6/10  ⚠️ 実装と乖離あり

総合評価: 6.9/10 (良好だが改善余地あり)
```

### **重要な発見**

#### ✅ **強み**
1. Box-First設計が確立（責務分離明確）
2. 段階的移行戦略（Legacy→New Extract Boxes）
3. 循環依存なし（健全な依存グラフ）
4. Fail-Fast設計（エラーハンドリング明確）

#### ⚠️ **弱み**
1. 超大型ファイル2つ（921行, 547行）
2. スタブディレクトリ3つ（66行の技術的負債）
3. SSA実装の重複（builder/ vs pipeline_v2/）
4. テスト不足（tests/に実装なし）

---

## 🚨 **最優先アクション（今すぐできる）**

### **Phase 1: クリーンアップ** 🔴
```bash
# 1. スタブディレクトリ削除 (影響小、効果大)
cd apps/selfhost-compiler
rm -rf mir/ parser/ emitter/

# 削除対象:
# - mir/ (20行) - 全ファイルが5行スタブ
# - parser/ (30行) - 全ファイルが5行スタブ
# - emitter/ (16行) - 全ファイルが8行スタブ
```

### **Phase 2: リファクタリング** 🟡
**優先度1**: `boxes/parser_box.hako` (921行) 分割
```
分割案:
- lexer.hako (200行) - トークン走査
- parser.hako (400行) - AST構築
- json_emitter.hako (200行) - JSON生成
- parser_helpers.hako (121行) - ヘルパー関数
```

**優先度2**: `builder/ssa/local.nyash` (547行) 分割
```
分割案:
- copy_insertion.nyash (150行) - Copy挿入
- phi_handling.nyash (150行) - PHI処理
- value_tracking.nyash (150行) - 値追跡
- helpers.nyash (97行) - ヘルパー関数
```

---

## 📊 **ディレクトリ別評価**

| Directory | Files | Lines | Status | 評価 | Next Action |
|-----------|-------|-------|--------|------|-------------|
| **pipeline_v2/** | 24 | 2,045 | ✅ 実装済 | A+ | 健全、現状維持 |
| **boxes/** | 7 | 2,038 | ✅ 実装済 | B+ | parser_box分割 |
| **builder/** | 11 | 886 | ⚠️ 一部スタブ | B | local.nyash分割 |
| **emitter/** | 2 | 16 | ❌ スタブ | F | 削除 |
| **mir/** | 4 | 20 | ❌ スタブ | F | 削除 |
| **parser/** | 6 | 30 | ❌ スタブ | F | 削除 |
| **tests/** | 0 | 0 | ❌ 未実装 | F | テスト追加 |

---

## 🏗️ **アーキテクチャ図（簡易版）**

```
                ┌──────────────┐
                │ compiler.hako│
                │   (Main)     │
                └───────┬──────┘
                        │
        ┌───────────────┴───────────────┐
        │                               │
        ▼                               ▼
┌──────────────┐              ┌──────────────┐
│ ParserBox    │              │ PipelineV2   │
│ (921 lines)  │              │ (382 lines)  │
│   ⚠️ 要分割    │              └───────┬──────┘
└──────────────┘                      │
                        ┌─────────────┼─────────────┐
                        │             │             │
                        ▼             ▼             ▼
                ┌──────────┐  ┌──────────┐  ┌──────────┐
                │ Extract  │  │   Emit   │  │ Utility  │
                │ (4 boxes)│  │ (6 boxes)│  │ (4 boxes)│
                └──────────┘  └──────────┘  └──────────┘
```

---

## 📖 **レポート活用ガイド**

### **ケース1: 全体像を素早く把握したい**
1. **START**: `selfhost_compiler_summary.md` を読む (5分)
2. 興味ある部分を `selfhost_compiler_structure.md` で深掘り (15分)

### **ケース2: リファクタリング計画を立てたい**
1. `selfhost_compiler_structure.md` の「推奨アクション」セクション
2. `selfhost_compiler_dependency_graph.txt` で影響範囲確認
3. 優先度マトリックスに基づいて実施順序決定

### **ケース3: 特定モジュールの依存関係を知りたい**
1. `selfhost_compiler_dependency_graph.txt` で該当Boxを検索
2. 依存ツリーをたどる
3. 外部依存セクションで追加依存を確認

### **ケース4: 技術的負債を洗い出したい**
1. `selfhost_compiler_summary.md` の「問題点TOP5」
2. `selfhost_compiler_structure.md` の「問題点・改善候補」
3. `selfhost_compiler_dependency_graph.txt` の「LEGACY/STUB MODULES」

---

## 🎓 **重要な洞察**

### **1. Box-First設計の成功**
- Extract → Normalize → Emit の3層分離が明確
- 各Boxが単一責務を持つ
- Flow/Box分離で制御フローと機能が独立

### **2. 段階的移行戦略**
- Legacy (`Stage1ExtractFlow`) → New (Extract Boxes)
- Fallback機構で後方互換性維持
- 技術的負債を増やさない設計

### **3. 改善の余地**
- スタブ（66行）は即座に削除可能
- 超大型ファイル（1,468行）は分割必須
- テストインフラは優先構築

---

## 📋 **チェックリスト**

### **即座に実施可能（今日）** ✅
- [ ] スタブディレクトリ削除 (`mir/`, `parser/`, `emitter/`)
- [ ] `interfaces.hako` 更新（実装と同期）
- [ ] README更新（Pipeline v2設計反映）

### **短期（1週間以内）** 🟡
- [ ] `parser_box.hako` 分割計画立案
- [ ] `local.nyash` 分割計画立案
- [ ] Extract層統一ロードマップ作成

### **中期（1ヶ月以内）** 🟢
- [ ] 超大型ファイル分割実施
- [ ] SSA層統合計画立案
- [ ] テストインフラ構築開始

---

## 🔗 **関連ドキュメント**

### **プロジェクト内**
- `apps/selfhost-compiler/README.md` - コンパイラ概要
- `apps/selfhost-compiler/INTERFACES.md` - インターフェース定義
- `apps/selfhost-compiler/pipeline_v2/README.md` - Pipeline v2設計

### **上位ドキュメント**
- `docs/development/selfhosting/pipeline_v2.md` - Pipeline v2詳細設計
- `docs/development/roadmap/phases/phase-15/` - Phase 15計画

---

## 📞 **質問・フィードバック**

このレポートに関する質問や、追加調査が必要な項目があれば、以下を参照:

1. **アーキテクチャ質問**: `selfhost_compiler_structure.md` の該当セクション
2. **依存関係の詳細**: `selfhost_compiler_dependency_graph.txt`
3. **優先度判断**: `selfhost_compiler_summary.md` の「推奨アクション」

---

**調査完了日**: 2025-10-04
**調査者**: Claude (Anthropic)
**調査手法**: ファイル構造分析、依存関係追跡、コード品質評価
