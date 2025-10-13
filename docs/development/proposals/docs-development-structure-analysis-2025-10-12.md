# docs/development/ 構造分析レポート

**分析日**: 2025-10-12
**対象**: `/home/tomoaki/git/hakorune-selfhost/docs/development/`
**目的**: ディレクトリ構造の整理・統合・削減の余地を調査

---

## 📊 全体統計

- **サブディレクトリ数**: 119個
- **トップレベルディレクトリ数**: 30個
- **総ファイル数**: 504個（全体）、462個（.mdファイル）
- **総行数**: 123,288行（.mdファイル合計）
- **階層深度**: 最大6階層（development/ を1階層目として）

---

## 🗂️ ディレクトリ構造概要

### トップレベルディレクトリ一覧（ファイル数・行数順）

| ディレクトリ | ファイル数 | 行数 | 備考 |
|-------------|-----------|------|------|
| **roadmap** | 223 | 58,526 | Phase別実装計画 |
| **proposals** | 72 | 23,554 | 設計提案・アイデア |
| **analysis** | 33 | 11,793 | 分析レポート ⚠️ ai-* 違反含む |
| **current** | 33 | 7,128 | 現在のタスク |
| investigations | 8 | 4,334 | 調査レポート |
| current_task_archive | 2 | 3,918 | タスクアーカイブ（最近更新） |
| archive | 13 | 3,065 | アーカイブ |
| architecture | 11 | 2,397 | アーキテクチャ設計 |
| issues | 10 | 1,799 | 問題・バグ調査 |
| refactoring | 3 | 1,798 | リファクタリング関連 |
| enum | 2 | 671 | enum機能関連 |
| strategies | 4 | 651 | 開発戦略 |
| cleanup | 4 | 503 | クリーンアップ関連 |
| benchmarks | 2 | 488 | ベンチマーク |
| design | 7 | 487 | 設計ドキュメント |
| philosophy | 3 | 392 | 設計哲学 |
| testing | 4 | 321 | テスト関連 |
| selfhosting | 5 | 308 | セルフホスト関連 |
| builder | 3 | 197 | ビルダー関連 |
| mir | 4 | 140 | MIR関連 |
| status | 1 | 130 | ステータスレポート |
| runtime | 1 | 90 | ランタイム関連 |
| engineering | 2 | 79 | エンジニアリング |
| migration | 1 | 77 | マイグレーション |
| adr | 1 | 70 | Architecture Decision Record |
| notes | 2 | 64 | 開発ノート |
| vm_ops | 1 | 45 | VM操作関連 |
| plan | 1 | 33 | 計画 |
| abi | 1 | 32 | ABI関連 |
| tools | 2 | 26 | ツール関連 |

---

## ⚠️ 問題点

### 1. 空ディレクトリ（5個）

完全に空（ファイル・サブディレクトリなし）のディレクトリ：

1. `roadmap/phases/phase-11.9/archive/` ← 削除推奨
2. `roadmap/phases/phase-15/archive/` ← 削除推奨
3. `testing/golden/` ← 削除推奨
4. `proposals/ideas/refactoring/selfhost-super-refactoring/analysis/` ← 削除推奨
5. `proposals/ideas/refactoring/selfhost-super-refactoring/plans/` ← 削除推奨

**削減見込み**: 5ディレクトリ（構造整理）

---

### 2. 単一ファイルディレクトリ（28個）

1つのファイルしか含まないディレクトリ（親ディレクトリへの統合候補）：

#### 即座統合推奨（小規模・明確）

| ディレクトリ | ファイル | 行数 | 統合先候補 |
|-------------|---------|------|-----------|
| `runtime/` | ENV_VARS.md | 90 | `docs/reference/` または `docs/guides/` |
| `status/` | phase24-verification-report.md | 130 | `roadmap/phases/phase-24/` |
| `adr/` | adr-001-no-corebox-everything-is-plugin.md | 70 | `architecture/` |
| `plan/` | plugin-abi-final-rollout.md | 33 | `roadmap/` または `architecture/` |
| `migration/` | mir-call-unification.md | 77 | `architecture/` または `roadmap/` |
| `vm_ops/` | registry-ssot.md | 45 | `architecture/` |
| `abi/` | host_api.md | 32 | `architecture/` |
| `proposals/concurrency/` | boxes.md | 77 | `proposals/` 直下 |
| `current/archive/` | claude_task_20251011.md | 429 | `current_task_archive/` |
| `current/selfhost/` | dep_tree_min_string.md | 39 | `current/` 直下または `selfhosting/` |

#### 検討要（サブディレクトリ構造の一部）

| ディレクトリ | ファイル | 行数 | 備考 |
|-------------|---------|------|------|
| `analysis/ai-discussions-2025-08/` | 2025-08-29-ancp-gemini-codex-analysis.md | 136 | ⚠️ ai-* 違反 |
| `architecture/runner/` | entry-resolve-box.md | 96 | 構造維持でOK |
| `design/blueprints/` | strings-utf8-byte.md | 61 | 構造維持でOK |
| `roadmap/phases/phase-14/` | phase14_packaging_ci_polish.md | 24 | Phase構造維持 |
| `roadmap/phases/phase-16/` | README.md | 66 | Phase構造維持 |
| `roadmap/phases/phase-19/` | README.md | 34 | Phase構造維持 |
| `roadmap/phases/phase-12.7/` | README.md | 452 | Phase構造維持（サブディレクトリあり） |
| `roadmap/phases/phase-12.7/grammar-specs/` | README.md | 71 | 構造維持 |
| `roadmap/phases/phase-15.9/` | README.md | 99 | Phase構造維持 |
| `roadmap/phases/phase-18/` | README.md | 39 | Phase構造維持 |
| `roadmap/phases/phase-15/phase-15.1/` | README.md | 88 | 構造維持 |
| `roadmap/phases/phase-20-python-integration/documentation/` | README.md | 270 | 構造維持 |
| `roadmap/phases/phase-20-python-integration/testing/` | README.md | 263 | 構造維持 |
| `roadmap/idea/` | README.md | 76 | `proposals/ideas/` に統合検討 |
| `roadmap/mir/core-13/step-50/` | README.md | 25 | 構造維持 |
| `proposals/ideas/` | plugin-box-conversion-candidates.md | 633 | サブディレクトリあり、維持 |
| `proposals/ideas/language/` | pure-functional-blocks.md | 157 | 構造維持 |
| `proposals/ideas/new-features/` | neural-mir-latent-vector.md | 331 | 構造維持 |

**削減見込み**: 10ディレクトリ統合 + 構造整理

---

### 3. CLAUDE.md ルール違反（3個）

`ai-*`, `claude-*`, `analysis-*` パターンのディレクトリ：

1. **`analysis/ai-discussions-2025-08/`** ← ⚠️ 違反
   - ファイル: `2025-08-29-ancp-gemini-codex-analysis.md` (136行)
   - 推奨: `analysis/discussions-2025-08/` にリネーム

2. **`analysis/phase-12-7-ai-feedback/`** ← ⚠️ 違反
   - ファイル: 5個、1,109行
   - 推奨: `analysis/phase-12-7-feedback/` にリネーム

3. **`analysis/phase-21-ai-evaluation/`** ← ⚠️ 違反
   - ファイル: 10個、5,118行
   - 推奨: `analysis/phase-21-evaluation/` にリネーム

**削減見込み**: 3ディレクトリ リネーム（構造整理）

---

### 4. アーカイブ肥大化（11個のアーカイブ関連ディレクトリ）

| ディレクトリ | ファイル数 | 行数 | 状態 |
|-------------|-----------|------|------|
| `current_task_archive/` | 2 | 3,918 | 🟢 最近更新（2025-10-11/12） |
| `archive/` | 13 | 3,065 | 🟡 要確認 |
| `archive/sessions/` | 11 | 2,186 | 🟡 要確認 |
| `roadmap/phases/phase-12/archive/` | 17 | 2,857 | 🟡 要確認 |
| `roadmap/phases/phase-12/archive/legacy-abi-docs/` | 4 | 680 | 🟡 要確認 |
| `roadmap/phases/phase-12.7/archive/` | 4 | 1,322 | 🟡 要確認 |
| `roadmap/native-plan/archives/` | 4 | 1,510 | 🟡 要確認 |
| `current/archive/` | 1 | 429 | → `current_task_archive/` に統合 |
| **`roadmap/phases/phase-11.9/archive/`** | 0 | 0 | 🔴 空（削除推奨） |
| **`roadmap/phases/phase-15/archive/`** | 0 | 0 | 🔴 空（削除推奨） |
| **`testing/golden/`** | 0 | 0 | 🔴 空（削除推奨） |

**削減見込み**:
- 空アーカイブ3個削除
- `current/archive/` 統合（429行）
- 古いアーカイブの精査で数千行削減可能

---

### 5. 重複・類似ディレクトリ（要統合検討）

#### A. analysis vs proposals vs investigations

| ディレクトリ | ファイル数 | 行数 | 用途 |
|-------------|-----------|------|------|
| `analysis/` | 33 | 11,793 | 分析レポート |
| `proposals/` | 72 | 23,554 | 設計提案・アイデア |
| `investigations/` | 8 | 4,334 | 調査レポート |
| `issues/` | 10 | 1,799 | 問題・バグ調査 |

**問題点**:
- `analysis/` と `investigations/` の違いが不明確
- `issues/` も調査内容を含む可能性

**推奨**:
- `investigations/` → `analysis/investigations/` に統合
- `issues/` → `analysis/issues/` に統合
- または、3つを統合して `analysis/` に集約

**削減見込み**: 2ディレクトリ統合、構造簡略化

#### B. current vs current_task_archive

| ディレクトリ | ファイル数 | 行数 | 更新日 |
|-------------|-----------|------|--------|
| `current/` | 33 | 7,128 | - |
| `current_task_archive/` | 2 | 3,918 | 2025-10-11/12 |
| `current/archive/` | 1 | 429 | 2025-10-11 |

**推奨**:
- `current/archive/` → `current_task_archive/` に統合（1ファイル、429行）

**削減見込み**: 1ディレクトリ統合

#### C. design vs architecture vs adr

| ディレクトリ | ファイル数 | 行数 | 用途 |
|-------------|-----------|------|------|
| `architecture/` | 11 | 2,397 | アーキテクチャ設計 |
| `design/` | 7 | 487 | 設計ドキュメント |
| `adr/` | 1 | 70 | Architecture Decision Record |

**推奨**:
- `adr/` → `architecture/adr/` に統合（1ファイル、70行）
- `design/` → `architecture/design/` に統合検討

**削減見込み**: 1-2ディレクトリ統合

#### D. その他単独ディレクトリ

| ディレクトリ | ファイル数 | 行数 | 統合候補 |
|-------------|-----------|------|---------|
| `plan/` | 1 | 33 | `roadmap/` または `architecture/` |
| `strategies/` | 4 | 651 | `roadmap/` または `proposals/` |
| `refactoring/` | 3 | 1,798 | `proposals/refactoring/` に統合 |
| `philosophy/` | 3 | 392 | `docs/` 直下または `docs/guides/` |
| `notes/` | 2 | 64 | `archive/` または削除 |

**削減見込み**: 5ディレクトリ統合

---

### 6. 階層が深すぎる（3階層以上の深いディレクトリ）

**最深6階層（development/ = 1階層目）**:

1. `roadmap/phases/phase-12/discussions/nyash-abi-discussion/` (6階層)
2. `roadmap/phases/phase-12/discussions/abi-strategy-discussion/` (6階層)
3. `roadmap/phases/phase-12/archive/legacy-abi-docs/` (6階層)
4. `proposals/ideas/refactoring/selfhost-super-refactoring/reports/` (6階層)
5. `proposals/ideas/refactoring/selfhost-super-refactoring/plans/` (6階層) ← 🔴 空
6. `proposals/ideas/refactoring/selfhost-super-refactoring/analysis/` (6階層) ← 🔴 空

**問題点**:
- 6階層は深すぎる（ナビゲーション困難）
- `selfhost-super-refactoring/` 配下に空ディレクトリ2個

**推奨**:
- Phase-12関連は構造維持（歴史的価値）
- `selfhost-super-refactoring/` 配下の空ディレクトリ削除
- 今後は4階層以内を目標

---

## 📈 TOP10 ランキング

### ファイル数 TOP10

1. **roadmap** - 223ファイル
2. **proposals** - 72ファイル
3. **current** - 33ファイル
4. **analysis** - 33ファイル
5. **archive** - 13ファイル
6. **architecture** - 11ファイル
7. **issues** - 10ファイル
8. **investigations** - 8ファイル
9. **design** - 7ファイル
10. **selfhosting** - 5ファイル

### 総行数 TOP10

1. **roadmap** - 58,526行（47.5%）
2. **proposals** - 23,554行（19.1%）
3. **analysis** - 11,793行（9.6%）
4. **current** - 7,128行（5.8%）
5. **investigations** - 4,334行（3.5%）
6. **current_task_archive** - 3,918行（3.2%）
7. **archive** - 3,065行（2.5%）
8. **architecture** - 2,397行（1.9%）
9. **issues** - 1,799行（1.5%）
10. **refactoring** - 1,798行（1.5%）

**注**: roadmap + proposals で全体の66.6%を占める

---

## 🎯 整理提案

### 優先度1: 即座実施可能（高効果・低リスク）

#### 1-1. 空ディレクトリ削除（5個）
```bash
rm -rf docs/development/roadmap/phases/phase-11.9/archive
rm -rf docs/development/roadmap/phases/phase-15/archive
rm -rf docs/development/testing/golden
rm -rf docs/development/proposals/ideas/refactoring/selfhost-super-refactoring/analysis
rm -rf docs/development/proposals/ideas/refactoring/selfhost-super-refactoring/plans
```
**削減**: 5ディレクトリ

#### 1-2. CLAUDE.md ルール違反リネーム（3個）
```bash
mv docs/development/analysis/ai-discussions-2025-08 \
   docs/development/analysis/discussions-2025-08

mv docs/development/analysis/phase-12-7-ai-feedback \
   docs/development/analysis/phase-12-7-feedback

mv docs/development/analysis/phase-21-ai-evaluation \
   docs/development/analysis/phase-21-evaluation
```
**削減**: 3ディレクトリ リネーム

#### 1-3. 単一ファイルディレクトリ統合（10個、高優先度）

| 元の場所 | ファイル | 移動先 |
|---------|---------|-------|
| `runtime/` | ENV_VARS.md | `docs/reference/environment-variables.md` にマージ |
| `status/` | phase24-verification-report.md | `roadmap/phases/phase-24/` |
| `adr/` | adr-001-no-corebox-everything-is-plugin.md | `architecture/adr/` |
| `plan/` | plugin-abi-final-rollout.md | `architecture/plans/` |
| `migration/` | mir-call-unification.md | `architecture/migrations/` |
| `vm_ops/` | registry-ssot.md | `architecture/` |
| `abi/` | host_api.md | `architecture/abi/` |
| `proposals/concurrency/` | boxes.md | `proposals/` 直下 |
| `current/archive/` | claude_task_20251011.md | `current_task_archive/` |
| `current/selfhost/` | dep_tree_min_string.md | `current/` 直下 |

**削減**: 10ディレクトリ統合、約1,000行整理

---

### 優先度2: 要検討（中効果・中リスク）

#### 2-1. analysis系ディレクトリ統合
```
analysis/               (33ファイル, 11,793行)
  ├── investigations/   (統合: 8ファイル, 4,334行)
  └── issues/           (統合: 10ファイル, 1,799行)
```
**削減**: 2ディレクトリ統合、17,926行を analysis/ 配下に集約

#### 2-2. 小規模ディレクトリ統合
```
architecture/
  ├── adr/              (統合: 1ファイル, 70行)
  ├── abi/              (統合: 1ファイル, 32行)
  └── migrations/       (統合: migration/, 1ファイル, 77行)

roadmap/
  └── strategies/       (統合: 4ファイル, 651行)

proposals/
  └── refactoring/      (統合: refactoring/, 3ファイル, 1,798行)
```
**削減**: 5ディレクトリ統合、2,628行整理

#### 2-3. notes/ とphilosophy/ の移動
```
docs/guides/
  └── development-philosophy.md  (統合: philosophy/, 392行)

archive/
  └── notes/                      (統合: notes/, 64行)
```
**削減**: 2ディレクトリ統合、456行整理

---

### 優先度3: 慎重検討（高効果・高リスク）

#### 3-1. roadmap/phases/ の古いアーカイブ削減

対象:
- `roadmap/phases/phase-12/archive/` (17ファイル, 2,857行)
- `roadmap/phases/phase-12.7/archive/` (4ファイル, 1,322行)
- `roadmap/native-plan/archives/` (4ファイル, 1,510行)

**推奨**:
- Phase-12/12.7は完了済み → 歴史的価値確認後、削減検討
- native-plan/archives は統合または削減

**削減見込み**: 3,000-5,000行

#### 3-2. archive/ 全体の見直し
```
archive/                (13ファイル, 3,065行)
  └── sessions/         (11ファイル, 2,186行)
```

**推奨**:
- 6ヶ月以上前のファイル → 削除検討
- sessions/ は開発ログ → 必要性確認

**削減見込み**: 1,000-2,000行

---

## 💾 削減見込みサマリー

### 即座実施可能（優先度1）
- **空ディレクトリ削除**: 5個
- **ルール違反リネーム**: 3個
- **単一ファイル統合**: 10個
- **行数整理**: 約1,000行

### 要検討（優先度2）
- **analysis系統合**: 2ディレクトリ（17,926行集約）
- **小規模統合**: 5ディレクトリ（2,628行整理）
- **notes/philosophy移動**: 2ディレクトリ（456行整理）

### 慎重検討（優先度3）
- **古いアーカイブ削減**: 3,000-5,000行
- **archive/見直し**: 1,000-2,000行

### **総削減可能**
- **ディレクトリ**: 27個削減（119個 → 92個、23%減）
- **行数**: 8,000-12,000行削減（123,288行 → 111,000-115,000行、7-10%減）
- **構造改善**: 階層簡略化、CLAUDE.mdルール準拠

---

## 🔍 補足分析

### proposals/ideas/ 内訳

| サブディレクトリ | ファイル数 | 行数 | 備考 |
|----------------|-----------|------|------|
| `ideas/improvements/` | 21 | 6,565 | 80/20ルール残り20%改善候補 |
| `ideas/refactoring/` | 20 | 8,107 | リファクタリング提案 |
| `ideas/tools/` | 4 | 494 | ツール改善 |
| `ideas/new-features/` | 1 | 331 | 新機能アイデア |
| `ideas/language/` | 1 | 157 | 言語機能提案 |

**合計**: 47ファイル、15,654行（proposals/の66%）

### current/ 内訳

| サブディレクトリ | ファイル数 | 用途 |
|----------------|-----------|------|
| `current/main/` | 多数 | メイン開発タスク |
| `current/llvm/` | 多数 | LLVMバックエンドタスク |
| `current/wasm/` | 多数 | WASMバックエンドタスク |
| `current/selfhost/` | 1 | セルフホストタスク（統合候補） |
| `current/self_current_task/` | 多数 | セルフホスト関連タスク |
| `current/archive/` | 1 | アーカイブ（統合候補） |

---

## 📋 実施チェックリスト

### Phase 1: 即座実施（所要時間: 30分）
- [ ] 空ディレクトリ5個削除
- [ ] CLAUDE.mdルール違反3個リネーム
- [ ] `current/archive/` → `current_task_archive/` 統合
- [ ] `proposals/concurrency/boxes.md` → `proposals/` 直下

### Phase 2: 構造整理（所要時間: 2時間）
- [ ] `adr/` → `architecture/adr/` 統合
- [ ] `abi/` → `architecture/abi/` 統合
- [ ] `plan/` → `architecture/plans/` 統合
- [ ] `migration/` → `architecture/migrations/` 統合
- [ ] `vm_ops/registry-ssot.md` → `architecture/` 統合
- [ ] `runtime/ENV_VARS.md` → `docs/reference/environment-variables.md` マージ
- [ ] `status/phase24-verification-report.md` → `roadmap/phases/phase-24/`

### Phase 3: 大規模統合（所要時間: 4時間）
- [ ] `investigations/` → `analysis/investigations/` 統合
- [ ] `issues/` → `analysis/issues/` 統合
- [ ] `refactoring/` → `proposals/refactoring/` 統合
- [ ] `strategies/` → `roadmap/strategies/` 統合
- [ ] `philosophy/` → `docs/guides/development-philosophy.md`
- [ ] `notes/` → `archive/notes/` または削除

### Phase 4: アーカイブ精査（所要時間: 要確認）
- [ ] `archive/` 6ヶ月以上前のファイル削除検討
- [ ] `roadmap/phases/phase-12/archive/` 削減検討
- [ ] `roadmap/phases/phase-12.7/archive/` 削減検討
- [ ] `roadmap/native-plan/archives/` 統合または削減

---

## 🎯 推奨アクション

### 今すぐ実施すべき（リスクゼロ）
1. **空ディレクトリ5個削除**（5分）
2. **CLAUDE.mdルール違反3個リネーム**（5分）
3. **`current/archive/` 統合**（5分）

**合計**: 15分で10個のディレクトリを整理

### 次に実施すべき（低リスク）
4. **単一ファイル統合10個**（1-2時間）
5. **小規模ディレクトリ統合5個**（1-2時間）

**合計**: 2-4時間で15個のディレクトリを整理、1,000行削減

### 慎重に検討すべき（要議論）
6. **analysis系統合**（4,000行以上の移動）
7. **古いアーカイブ削減**（歴史的価値の判断必要）

---

## 📝 備考

- **roadmap/** が全体の47.5%を占める → Phase別構造は維持推奨
- **proposals/** が19.1%、特に `ideas/` が多い → 80/20ルール適用中
- **current_task_archive/** は最近更新（2025-10-11/12） → 削減不要
- **階層6個は深すぎる** → 今後は4階層以内を目標
- **CLAUDE.mdルール違反3個** → 早急に修正推奨

---

**分析完了**: 2025-10-12
**分析者**: Claude (Task Agent)
**分析時間**: 約10分
