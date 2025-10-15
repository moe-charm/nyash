# Selfhost Compiler 構造調査 - クイックサマリー

**詳細レポート**: `/tmp/selfhost_compiler_structure.md` (655行)

---

## 📊 **一目でわかる統計**

```
総ファイル数: 57
総行数:      5,388
Box定義数:    30
Flow定義数:    5
ディレクトリ: 10

最大ファイル: parser_box.hako (921行)  ← 要分割
次点:         local.nyash (547行)      ← 要分割
```

---

## 🏗️ **アーキテクチャ概要**

### **3層構造**
```
1. Parser層    → Stage-1 JSON生成
2. Pipeline層  → Extract → Normalize → Emit
3. Backend層   → MIR実行 (VM/LLVM)
```

### **Pipeline v2の中核設計**
```
Extract Boxes (4個)  →  Normalizer  →  Emit Boxes (6個)
                              ↓
                          LocalSSA
                              ↓
                          MIR JSON
```

---

## ⚠️ **重要な問題点TOP5**

### 🔴 **1. 超大型ファイル（要分割）**
- `parser_box.hako` (921行) - パーサー・レキサー・JSON生成が混在
- `local.nyash` (547行) - LocalSSA実装が単一ファイルに集約

### 🔴 **2. スタブディレクトリ（削除候補）**
- `mir/` (20行) - 全ファイルが5行スタブ
- `parser/` (30行) - 全ファイルが5行スタブ
- `emitter/` (16行) - 全ファイルが8行スタブ

### 🟡 **3. 実装の重複**
- SSA層: `builder/ssa/` vs `pipeline_v2/local_ssa_box.hako`
- Extract層: `Stage1ExtractFlow` (レガシー) vs 新Extract Boxes

### 🟡 **4. テスト不足**
- `tests/` ディレクトリ存在するが実装ファイルなし
- ユニットテストが未整備

### 🟢 **5. ドキュメント更新**
- `interfaces.hako` と実装の乖離
- READMEが現在の設計と不整合

---

## 📋 **推奨アクションプラン**

### **Phase 1: クリーンアップ（即実施）** 🔴
1. スタブディレクトリ削除 (`mir/`, `parser/`, `emitter/`)
2. 重複定義の整理

### **Phase 2: リファクタリング（短期）** 🟡
3. `parser_box.hako` 分割 (921行 → 3-4ファイル)
4. `local.nyash` 分割 (547行 → 3-4ファイル)

### **Phase 3: 統合（中期）** 🟡
5. Extract層統一 (`Stage1ExtractFlow` 廃止)
6. SSA層統合 (`builder/ssa/` ↔ `pipeline_v2/`)

### **Phase 4: 品質向上（長期）** 🟢
7. テストインフラ構築
8. ドキュメント整備

---

## 🎯 **アーキテクチャ評価**

### **強み** ✅
- ✅ **Box-First設計** - 責務分離が明確
- ✅ **段階的移行** - レガシー→新設計への移行計画
- ✅ **循環依存なし** - 健全な依存グラフ
- ✅ **Fail-Fast設計** - エラーハンドリング明確

### **弱み** ⚠️
- ⚠️ **ファイル粒度不均一** - 921行 vs 5行（スタブ）
- ⚠️ **スタブ過剰残存** - 3ディレクトリが完全スタブ
- ⚠️ **実装重複** - SSA層、Extract層
- ⚠️ **テスト不足** - tests/に実装なし

---

## 📂 **ディレクトリ別サマリー**

| Directory | Files | Lines | Status | Note |
|-----------|-------|-------|--------|------|
| **boxes/** | 7 | 2,038 | ✅ 実装済 | parser_box.hakoが921行（要分割） |
| **pipeline_v2/** | 24 | 2,045 | ✅ 実装済 | 中核層、健全な設計 |
| **builder/** | 11 | 886 | ⚠️ 一部スタブ | local.nyashが547行（要分割） |
| **emitter/** | 2 | 16 | ❌ スタブ | 削除候補 |
| **mir/** | 4 | 20 | ❌ スタブ | 削除候補 |
| **parser/** | 6 | 30 | ❌ スタブ | 削除候補 |
| **tests/** | 0 | 0 | ❌ 未実装 | テスト追加必要 |

---

## 🔗 **主要モジュール依存**

```
compiler.hako (Main)
    ├─→ ParserBox (boxes/) [921行]
    ├─→ JsonProgramBox (boxes/) [520行]
    ├─→ ExecutionPipelineBox (pipeline_v2/)
    └─→ PipelineV2 (pipeline_v2/) [382行]
            ├─→ Extract Boxes (4個)
            ├─→ Emit Boxes (6個)
            ├─→ NormalizerBox
            └─→ LocalSSA (builder/ssa/)
```

---

## 💡 **最優先タスク（今すぐできる）**

1. **スタブ削除** (影響小、効果大)
   ```bash
   rm -rf mir/ parser/ emitter/
   ```

2. **interfaces.hako 更新** (ドキュメント作業)
   - 現在の実装に合わせて仕様更新

3. **README更新** (ドキュメント作業)
   - Pipeline v2設計を反映

---

## 📊 **コード品質スコア**

| 項目 | スコア | 評価 |
|------|--------|------|
| 設計品質 | 8/10 | ✅ Box-First設計が優秀 |
| コード整理度 | 6/10 | ⚠️ スタブ・超大型ファイル残存 |
| 保守性 | 7/10 | ✅ 責務分離は明確だが改善余地 |
| テストカバレッジ | 2/10 | ❌ テスト未整備 |
| ドキュメント | 6/10 | ⚠️ 実装と乖離あり |

**総合評価**: 🟡 **良好だが改善余地あり (6.9/10)**

---

## 🎓 **学んだ教訓**

### **成功パターン**
1. **Extract → Normalize → Emit** の3層分離
2. **Flow/Box明確な分離** (制御 vs 機能)
3. **段階的移行戦略** (Legacy→New with Fallback)

### **改善パターン**
1. **スタブは早期削除** (技術的負債の蓄積防止)
2. **200-300行を目安に分割** (可読性・保守性)
3. **テストファースト** (tests/を最初に整備)

---

**結論**:
アーキテクチャは**優秀**。初期開発の痕跡（スタブ、大型ファイル）を整理すれば、**世界クラスのセルフホストコンパイラ**に進化可能。Phase 1（スタブ削除）は今すぐ実施推奨。

---

**詳細分析**: `/tmp/selfhost_compiler_structure.md` を参照
