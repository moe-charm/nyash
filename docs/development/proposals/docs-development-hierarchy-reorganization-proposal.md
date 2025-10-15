# docs/development/ ディレクトリ階層統合提案

**作成日**: 2025-10-12
**対象**: `/home/tomoaki/git/hakorune-selfhost/docs/development/`
**目的**: トップレベルディレクトリ削減・階層構造最適化

---

## 📊 現状分析サマリー

### 基本統計
- **トップレベルディレクトリ数**: 24個（多すぎる！）
- **総ファイル数**: 461個のMarkdownファイル
- **総ディレクトリ数**: 109個
- **最深階層**: 12階層（深すぎる！）

### トップレベルディレクトリ一覧（ファイル数順）

| ディレクトリ | ファイル数 | サイズ区分 | 統合候補 |
|------------|-----------|-----------|---------|
| roadmap | 223 | 超大 | - |
| proposals | 73 | 大 | - |
| analysis | 33 | 中 | ✅ |
| current | 32 | 中 | - |
| architecture | 16 | 中 | - |
| archive | 13 | 中 | - |
| issues | 10 | 小 | ✅ |
| investigations | 8 | 小 | ✅ |
| design | 7 | 小 | ✅ |
| selfhosting | 5 | 極小 | ✅ |
| cleanup | 4 | 極小 | ✅ |
| mir | 4 | 極小 | ✅ |
| strategies | 4 | 極小 | ✅ |
| testing | 4 | 極小 | ✅ |
| builder | 3 | 極小 | ✅ |
| current_task_archive | 3 | 極小 | ✅ |
| philosophy | 3 | 極小 | ✅ |
| refactoring | 3 | 極小 | ✅ |
| benchmarks | 2 | 極小 | ✅ |
| engineering | 2 | 極小 | ✅ |
| enum | 2 | 極小 | ✅ |
| notes | 2 | 極小 | ✅ |
| tools | 2 | 極小 | ✅ |

**問題点**:
- ✅ 極小ディレクトリ（≤5ファイル）が **14個** も存在
- ✅ トップレベル24個は多すぎる（推奨: 6-10個）
- ✅ 類似・重複用途のディレクトリが複数存在

---

## 🎯 統合提案（10案）

### A. 大カテゴリ統合案（優先度: 高）

#### 提案1: `analysis/` 配下に調査系を統合 ⭐最優先
**移動元**: `investigations/` (8ファイル), `issues/` (10ファイル)
**移動先**: `analysis/investigations/`, `analysis/issues/`
**理由**: すべて「分析・調査・問題解決」という同一用途
**削減**: トップレベル -2個
**リスク**: 低

**統合後構造**:
```
analysis/
  ├── investigations/  ← 旧 investigations/
  ├── issues/         ← 旧 issues/
  └── (既存の分析ドキュメント)
```

---

#### 提案2: `architecture/` 配下に設計系を統合
**移動元**: `design/` (7ファイル), `philosophy/` (3ファイル)
**移動先**: `architecture/design/`, `architecture/philosophy/`
**理由**: すべて「設計思想・アーキテクチャ」に関する内容
**削減**: トップレベル -2個
**リスク**: 低

**統合後構造**:
```
architecture/
  ├── design/      ← 旧 design/
  ├── philosophy/  ← 旧 philosophy/
  └── (既存のアーキテクチャドキュメント)
```

---

#### 提案3: `testing/` を `tools/` 配下に統合
**移動元**: `benchmarks/` (2ファイル), `testing/` (4ファイル)
**移動先**: `tools/testing/`, `tools/benchmarks/`
**理由**: すべて「開発ツール・テスト環境」に関する内容
**削減**: トップレベル -1個（testing/benchmarks統合、tools拡張）
**リスク**: 低

**統合後構造**:
```
tools/
  ├── testing/     ← 旧 testing/
  ├── benchmarks/  ← 旧 benchmarks/
  └── (その他のツール関連)
```

---

#### 提案4: `refactoring/` を `proposals/` 配下に統合
**移動元**: `refactoring/` (3ファイル), `cleanup/` (4ファイル)
**移動先**: `proposals/refactoring/`, `proposals/cleanup/`
**理由**: すでに `proposals/ideas/refactoring/` が存在、重複を解消
**削減**: トップレベル -2個
**リスク**: 低（proposals内に同名サブディレクトリあり、マージ必要）

**統合後構造**:
```
proposals/
  ├── ideas/
  │   └── refactoring/  ← 既存
  ├── refactoring/      ← 旧 refactoring/ を統合
  └── cleanup/          ← 旧 cleanup/
```

---

#### 提案5: `selfhosting/` を `roadmap/` 配下に統合
**移動元**: `selfhosting/` (5ファイル)
**移動先**: `roadmap/selfhosting/`
**理由**: セルフホスティングはロードマップの一部
**削減**: トップレベル -1個
**リスク**: 低

---

#### 提案6: `builder/` と `mir/` を `architecture/` 配下に統合
**移動元**: `builder/` (3ファイル), `mir/` (4ファイル)
**移動先**: `architecture/builder/`, `architecture/mir/`
**理由**: すべて「コア実装・アーキテクチャ」の技術ドキュメント
**削減**: トップレベル -2個
**リスク**: 低

---

### B. アーカイブ整理案（優先度: 中）

#### 提案7: `current_task_archive/` を `archive/` 配下に統合
**移動元**: `current_task_archive/` (3ファイル)
**移動先**: `archive/current_task/`
**理由**: すでに `archive/` が存在、アーカイブは一元管理すべき
**削減**: トップレベル -1個
**リスク**: 低

---

### C. その他の統合案（優先度: 低）

#### 提案8: `strategies/` を `proposals/` または `architecture/` 配下に統合
**移動元**: `strategies/` (4ファイル)
**移動先**: `proposals/strategies/` または `architecture/strategies/`
**理由**: 戦略ドキュメントは提案またはアーキテクチャの一部
**削減**: トップレベル -1個
**リスク**: 中（内容確認が必要）

---

#### 提案9: `engineering/` を `architecture/` 配下に統合
**移動元**: `engineering/` (2ファイル)
**移動先**: `architecture/engineering/`
**理由**: エンジニアリング原則はアーキテクチャの一部
**削減**: トップレベル -1個
**リスク**: 低

---

#### 提案10: `enum/` と `notes/` を `proposals/ideas/` 配下に統合
**移動元**: `enum/` (2ファイル), `notes/` (2ファイル)
**移動先**: `proposals/ideas/enum/`, `proposals/ideas/notes/`
**理由**: 極小ディレクトリをアイデア集に統合
**削減**: トップレベル -2個
**リスク**: 低

---

## 📉 削減見込み

### 統合実行シナリオ

| シナリオ | 削減案 | トップレベル削減数 | 削減後の数 |
|---------|-------|------------------|----------|
| **最小限** | 提案1,2,7 | -5個 | 19個 |
| **推奨** | 提案1-7 | -11個 | 13個 |
| **最大限** | 提案1-10 | -15個 | 9個 ⭐目標 |

### 推奨統合順序（リスク順）

1. **Phase 1（低リスク）**: 提案1,2,3,5,6,7,9
   - 削減: -10個
   - 統合後: 14個

2. **Phase 2（要確認）**: 提案4,8,10
   - 削減: -5個
   - 統合後: 9個 ⭐目標達成

---

## 🎯 統合後の推奨トップレベル構造（9個）

```
docs/development/
├── analysis/           ← 統合: investigations, issues
├── architecture/       ← 統合: design, philosophy, builder, mir, engineering
├── archive/            ← 統合: current_task_archive
├── current/            ← 現状維持
├── proposals/          ← 統合: refactoring, cleanup, strategies, enum, notes
├── roadmap/            ← 統合: selfhosting
├── tools/              ← 統合: testing, benchmarks
└── (その他2-3個)
```

**削減**: 24個 → 9個（-15個、37.5%削減）

---

## ⚠️ 実行時の注意事項

### 高リスク項目
1. **proposals/refactoring/** との重複解消（提案4）
   - 既存の `proposals/ideas/refactoring/` と `refactoring/` をマージする必要あり
   - ファイル名重複チェック必須

2. **strategies/** の内容確認（提案8）
   - 提案なのか、アーキテクチャなのか、内容確認が必要

### 実行前チェックリスト
- [ ] 各ディレクトリの内部ファイル一覧を確認
- [ ] ファイル名重複チェック（特に proposals/refactoring/）
- [ ] 移動先ディレクトリの作成
- [ ] gitでの移動履歴保持（`git mv`使用）
- [ ] 相対パスリンクの破損チェック
- [ ] CLAUDE.md等のドキュメントハブ更新

---

## 🚀 次のアクション

### 推奨ステップ
1. **Phase 1実行**: 低リスク統合（提案1,2,3,5,6,7,9）
   - 期待削減: -10個（24個 → 14個）
   - 所要時間: 30-60分

2. **Phase 2実行**: 内容確認後統合（提案4,8,10）
   - 期待削減: -5個（14個 → 9個）
   - 所要時間: 30分

3. **検証**: リンク切れチェック・ビルド確認

### 即座に実行可能な最優先統合（5分で完了）

**提案1**: `investigations/` と `issues/` を `analysis/` 配下に移動
```bash
cd /home/tomoaki/git/hakorune-selfhost/docs/development
git mv investigations analysis/
git mv issues analysis/
```

**削減**: -2個（24個 → 22個）

---

## 📊 統合マトリックス（優先度×リスク）

| 提案 | 優先度 | リスク | 削減数 | 実行順序 |
|-----|-------|-------|-------|---------|
| 1 | 高 | 低 | -2 | 1 |
| 2 | 高 | 低 | -2 | 2 |
| 3 | 高 | 低 | -1 | 3 |
| 5 | 高 | 低 | -1 | 4 |
| 6 | 高 | 低 | -2 | 5 |
| 7 | 高 | 低 | -1 | 6 |
| 9 | 高 | 低 | -1 | 7 |
| 4 | 中 | 中 | -2 | 8 |
| 8 | 中 | 中 | -1 | 9 |
| 10 | 低 | 低 | -2 | 10 |

**合計削減**: -15個（24個 → 9個）

---

## 🎯 成功基準

- ✅ トップレベルディレクトリ: 24個 → 9個（62.5%削減）
- ✅ 極小ディレクトリ（≤5ファイル）: 14個 → 0個
- ✅ 階層構造: 2-3階層で統一（現在12階層）
- ✅ 用途重複ディレクトリ: 完全解消
- ✅ リンク切れ: 0件

---

**作成者**: Claude Code
**レビュー推奨**: Phase 1実行前にユーザー承認推奨
