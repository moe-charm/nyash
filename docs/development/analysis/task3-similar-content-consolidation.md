# Task 3: 類似コンテンツ統合候補レポート

**調査日時**: 2025-10-12
**対象**: `/home/tomoaki/git/hakorune-selfhost/docs` (1,289ファイル, 257,286行)
**目的**: 内容が似ている・重複するドキュメントの統合候補を特定

---

## 📊 エグゼクティブサマリー

### 主要発見
- **完全重複ディレクトリ**: 10個のフェーズがarchive/とactive/に二重存在
- **phase-12.7の完全重複**: 34ファイルすべてが完全一致（MD5検証済み）
- **バージョン違いファイル**: 9個（_v1, _v2, _old, _backup, _draft）
- **README重複**: 151個のREADME.md（多くは異なる内容だが整理可能）
- **トピック重複**: Phase 15関連212ファイル、Plugin関連39ファイル

### 削減可能性（推定）
- **即座に削除可能**: 約50-60ファイル（完全重複）
- **統合可能**: 約80-100ファイル（類似内容）
- **合計削減見込み**: 130-160ファイル（全体の10-12%）

---

## 🔴 **グループ1: 完全重複ディレクトリ（最優先）**

### 1.1 archive/phases と development/roadmap/phases の二重存在

**発見**: 以下10フェーズが両方のディレクトリに存在

```
phase-11.9
phase-12
phase-12.7  ← 完全一致（34ファイル、MD5検証済み）
phase-13
phase-14
phase-15
phase-15/phase-15.1
phase-16
phase-17
phase-18
phase-19
phase-21  ← 内容が異なる（archive版に追加あり）
phase-22
phase-50
```

#### 統合候補トップ3

**1. phase-12.7 (完全重複)**
- **archive**: `/home/tomoaki/git/hakorune-selfhost/docs/archive/phases/phase-12.7/` (34ファイル)
- **active**: `/home/tomoaki/git/hakorune-selfhost/docs/development/roadmap/phases/phase-12.7/` (34ファイル)
- **状況**: **完全一致**（MD5ハッシュ検証済み）
- **推奨**: active版を残し、archiveを削除
- **削減**: 34ファイル、452行（README.mdだけで）

**2. phase-21 (部分的に異なる)**
- **archive**: `/home/tomoaki/git/hakorune-selfhost/docs/archive/phases/phase-21/README.md` (406行)
- **active**: `/home/tomoaki/git/hakorune-selfhost/docs/development/roadmap/phases/phase-21/README.md` (499行)
- **差分**: active版に「Nyashの決定的優位性」「ファイルマッピング」「Box単位チェック」の追加セクション
- **推奨**: active版を残し、archive版のユニークな内容があれば移行後削除

**3. phase-12 (ヘッダー異なる)**
- **archive**: 旧版（Status noteなし）
- **active**: 新版（Phase-15 status note追加）
- **推奨**: active版を残し、archiveを削除

#### 統合アクション

```bash
# phase-12.7の完全削除（完全一致確認済み）
rm -rf docs/archive/phases/phase-12.7

# phase-21の差分確認後削除
# (active版が最新かつ完全なため)
rm docs/archive/phases/phase-21/README.md

# phase-12, 13, 15-19, 22, 50も確認後同様に処理
```

**削減見込み**: 約50ファイル

---

## 🟠 **グループ2: バージョン違いファイル**

### 2.1 発見されたバージョン違いファイル

```
1. docs/archive/phases/phase-21/README_v2.md
   → 統合先: phase-21/README.md（すでに統合済みと思われる）

2. docs/development/roadmap/phases/phase-21/README_v2.md
   → 要確認: active/README.mdとの差分チェック必要

3. docs/archive/specs-deprecated/language-specs/language_spec_old.md
   → 推奨: 削除（deprecatedフォルダ内）

4. docs/development/selfhosting/pipeline_v2.md
   → 確認: v1が存在するか？v2が最新版か？

5. docs/private/research/ai-dual-mode-development/workshop_paper_draft.md
   → 推奨: 最終版に統合後削除

6. docs/reference/abi/nyrt_c_abi_v0.md
   → 確認: 現行バージョンは何か？

7. docs/reference/ir/json_v0.md
   → 確認: 現行バージョンは何か？

8. docs/reference/jit/jit_stats_json_v1.md
   → 確認: v2が存在するか？

9. docs/reference/plugin-abi/nyash_abi_v2.md
   → 確認: これが最新版か？
```

#### 統合アクション

```bash
# Phase-21 v2ファイルの確認
diff docs/archive/phases/phase-21/README_v2.md docs/archive/phases/phase-21/README.md
diff docs/development/roadmap/phases/phase-21/README_v2.md docs/development/roadmap/phases/phase-21/README.md

# 旧言語仕様の削除（deprecated）
rm docs/archive/specs-deprecated/language-specs/language_spec_old.md

# draft削除（最終版があるか確認後）
# rm docs/private/research/ai-dual-mode-development/workshop_paper_draft.md
```

**削減見込み**: 5-9ファイル

---

## 🟡 **グループ3: トピック重複（Phase 15関連）**

### 3.1 Phase 15の分散状況

**Phase 15言及**: 212ファイル
**Phase 15主トピック**: 20ファイル

#### Phase 15アクティブファイル (32個)

```
docs/development/roadmap/phases/phase-15.13/README.md (265行)
docs/development/roadmap/phases/phase-15.15/README.md (203行)
docs/development/roadmap/phases/phase-15.5/README.md (137行)
docs/development/roadmap/phases/phase-15.7/README.md (1,350行) ← 最大
docs/development/roadmap/phases/phase-15.8/README.md (569行)
docs/development/roadmap/phases/phase-15.9/README.md (115行)
docs/development/roadmap/phases/phase-15/README.md (504行)
docs/development/roadmap/phases/phase-15/phase-15.1/README.md (267行)
... (全32ファイル)
```

#### Phase 15アーカイブファイル (13個)

```
docs/archive/phases/phase-15/README.md (385行)
docs/archive/phases/phase-15/phase-15.1/README.md (267行)
... (全13ファイル)
```

#### 統合候補

**サブフェーズの統合**:
- Phase 15.5, 15.7, 15.8, 15.9, 15.13, 15.15 → それぞれが独立したREADME
- **推奨**: 完了したサブフェーズは親フェーズ（phase-15/）のREADMEに統合
- **理由**: 212ファイルでPhase 15言及 → 情報が分散しすぎ

**実装戦略ドキュメントの統合**:
```
docs/development/roadmap/phases/phase-15/implementation/
├── lld-strategy.md
├── architecture.md
├── box-stacking.md
├── self-hosting-strategy-2025-09.md
└── ... (他7ファイル)
```
→ 推奨: 統合してphase-15/IMPLEMENTATION.mdにまとめる

**削減見込み**: 8-12ファイル（統合によるナビゲーション改善）

---

## 🟡 **グループ4: トピック重複（プラグインシステム）**

### 4.1 プラグイン関連ファイル (39個)

#### 重複パターン

**migration-guide系**:
```
docs/reference/architecture/plugin-migration-guide-v2.md (419行)
docs/reference/architecture/plugin-migration-guide-v2-summary.md (98行)
docs/reference/plugin-system/migration-guide.md (別内容?)
```
→ 推奨: v2を正式版として、summaryを統合

**Plugin ABI系**:
```
docs/reference/abi/nyrt_c_abi_v0.md
docs/reference/abi/PLUGIN_ABI.md
docs/reference/plugin-abi/nyash_abi_v2.md
```
→ 推奨: 現行バージョンを`docs/reference/plugin-system/`に統合

**削減見込み**: 3-5ファイル

---

## 🟢 **グループ5: Macro関連ファイル（整理推奨）**

### 5.1 Macroガイドファイル (5個)

```
docs/guides/macro-system.md (166行) ← クイックスタート
docs/guides/user-macros.md (182行) ← ユーザー向け詳細
docs/guides/macro-box.md (32行) ← MacroBox仕様
docs/guides/macro-box-nyash.md (53行) ← Nyash側API
docs/guides/macro-profiles.md (40行) ← プロファイル
```

#### 統合候補

**推奨構成**:
```
docs/guides/macro-guide.md  ← macro-system.md + user-macros.mdを統合
docs/reference/macro-system/
├── MacroBox.md  ← macro-box.md + macro-box-nyash.md
└── profiles.md  ← macro-profiles.md
```

**削減見込み**: 2-3ファイル（統合による）

---

## 🟢 **グループ6: WASM関連ファイル（整理推奨）**

### 6.1 WASM関連ファイル (13個)

```
docs/guides/wasm-guide/wasm_quick_start.md
docs/guides/wasm-guide/wasm_browser_plan.md
docs/guides/wasm-roadmap.md
docs/guides/wasm-abi.md
docs/guides/wasm-benchmarks.md
docs/development/current/wasm/wasm_boxification_handoff.md
docs/development/current/wasm/benchmark-implementation.md
... (他6個)
```

#### 統合候補

**推奨構成**:
```
docs/guides/wasm-guide/
├── README.md  ← クイックスタート統合
├── abi.md
├── benchmarks.md
└── browser-integration.md  ← browser_plan統合
```

**削減見込み**: 2-3ファイル

---

## 📈 **統計サマリー**

### ファイル統計
- **総markdownファイル**: 1,289
- **総行数**: 257,286行
- **README.md**: 151個
- **PLAN.md**: 11個
- **INDEX.md**: 6個

### ディレクトリサイズ
- **docs/archive**: 9.3MB
- **docs/development/roadmap/phases**: 2.4MB
- **docs/private**: 9.2MB

### 削減可能性サマリー

| グループ | 削減ファイル数 | 削減行数（推定） | 優先度 |
|---------|---------------|-----------------|--------|
| 1. 完全重複ディレクトリ | 50-60 | 5,000-8,000 | 🔴 最高 |
| 2. バージョン違い | 5-9 | 300-500 | 🟠 高 |
| 3. Phase 15統合 | 8-12 | 500-800 | 🟡 中 |
| 4. Plugin統合 | 3-5 | 200-400 | 🟡 中 |
| 5. Macro統合 | 2-3 | 100-200 | 🟢 低 |
| 6. WASM統合 | 2-3 | 100-200 | 🟢 低 |
| **合計** | **70-92** | **6,200-10,100** | - |

### トピック重複統計

| トピック | 言及ファイル数 | 主トピックファイル数 |
|---------|--------------|-------------------|
| Everything is Box | 388 | 50+ |
| Phase 15 | 212 | 20 |
| MIR命令 | 206 | 15 |
| セルフホスト | 31 | 5 |
| Plugin System | 22 | 15 |
| LLVM Backend | 19 | 10 |

---

## 🎯 **推奨アクション（優先順位順）**

### Phase 1: 完全重複削除（即実行可能）

```bash
# 1. phase-12.7の完全削除（MD5検証済み）
rm -rf docs/archive/phases/phase-12.7

# 2. 旧言語仕様の削除
rm docs/archive/specs-deprecated/language-specs/language_spec_old.md

# 3. phase-21等の差分確認後削除
# (手動確認後に実行)
```

**期待効果**: 35-40ファイル削減

### Phase 2: バージョン違いファイルの整理

```bash
# 各v1/v2/oldファイルの確認と統合
# (個別に内容確認必要)
```

**期待効果**: 5-9ファイル削減

### Phase 3: トピック別統合（Phase 15, Plugin, Macro, WASM）

**手順**:
1. Phase 15サブフェーズの統合戦略決定
2. Plugin migration-guide系の統合
3. Macroガイドの再構成
4. WASMガイドの再構成

**期待効果**: 15-23ファイル削減、ナビゲーション大幅改善

### Phase 4: archive/phases全体の見直し

**archive内の10フェーズ**を個別にactive版と比較:
- 完全一致 → 削除
- 部分一致 → ユニークな内容をactiveに移行後削除
- 異なる → archiveに残す（日付付きで明示）

**期待効果**: 20-30ファイル削減

---

## 🚨 **注意事項**

### 削除前の必須チェック
1. **Gitコミット履歴**: 削除前にgit logで履歴確認
2. **外部リンク**: 他ドキュメントからのリンク切れチェック
3. **ユニーク情報**: archive版にしかない情報の移行
4. **バックアップ**: 削除前にブランチ作成推奨

### 統合時のベストプラクティス
1. **日付記録**: 統合元ファイルの作成日を記録
2. **変更履歴**: どこから統合したか明記
3. **リダイレクト**: 可能ならGitHubのsymbolic linkで旧パスを保持

---

## 📝 **次回タスクへの引き継ぎ**

### Task 1 (重複ファイル統合) への情報
- phase-12.7が完全重複（最優先削除候補）
- phase-21等も高確率で統合可能

### Task 2 (ディレクトリ構造) への情報
- archive/phasesとactive/phasesの二重管理が非効率
- 統合後の明確な配置ルールが必要

### 全体最適化への提案
1. **archive/phases → archive/phases-old-YYYY-MM/** に日付別アーカイブ
2. **active版のみを正式版**とする明確なルール策定
3. **Phase完了時のアーカイブフロー**の確立

---

**調査完了**: 2025-10-12
**次回アクション**: Phase 1実行（完全重複削除）
