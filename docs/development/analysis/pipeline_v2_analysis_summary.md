# Pipeline v2 箱化分析 - エグゼクティブサマリー

**分析日**: 2025-10-12
**分析者**: Claude (Sonnet 4.5)
**対象**: apps/selfhost-compiler/pipeline_v2/ および common/

---

## 🎯 主要な発見（3つの重複パターン）

### 1️⃣ **Extract系の完全重複** ⭐最優先

**影響度**: 🔴 極めて高い（85%重複）

| 指標 | 値 |
|------|-----|
| 対象ファイル | 3ファイル (call/method/new_extract_box) |
| 重複行数 | 60-70行（30行が完全一致） |
| 削減見込み | 62行 (-40%) |
| 実装難易度 | 低（2時間） |

**アクション**: `Stage1IntArgsExtractBox` 新設で括弧深度追跡とInt抽出を共通化

---

### 2️⃣ **Normalizer系の重複** ⭐優先度高

**影響度**: 🟡 高い（85%重複）

| 指標 | 値 |
|------|-----|
| 対象ファイル | 1ファイル (normalizer_box.hako) |
| 重複行数 | 40行（16行が完全一致） |
| 削減見込み | 22行 (-23%) |
| 実装難易度 | 低（1.5時間） |

**アクション**: `_normalize_with_label_and_args()` 新設で3メソッドを薄いラッパーに

---

### 3️⃣ **Emit系の重複** ⭐優先度中

**影響度**: 🟢 中程度（50%重複）

| 指標 | 値 |
|------|-----|
| 対象ファイル | 3ファイル (emit_call/method/newbox_box) |
| 重複行数 | 25-30行 |
| 削減見込み | 27行 (-16%) |
| 実装難易度 | 低（1.5時間） |

**アクション**: `ArgsConstEmitBox` 新設で引数materialize処理を共通化

---

## 📊 総合削減効果

### Phase 1実装（推奨）

| 項目 | 削減 | 所要時間 |
|------|------|---------|
| Extract系統合 | 62行 | 2時間 |
| Normalizer共通化 | 22行 | 1.5時間 |
| Emit系統合 | 27行 | 1.5時間 |
| スモークテスト | - | 1時間 |
| **合計** | **111行 (-3.9%)** | **6時間** |

### 対象ファイルへの影響

| ファイル | Before | After | 削減率 |
|---------|--------|-------|-------|
| call_extract_box | 54行 | 24行 | **-56%** |
| method_extract_box | 51行 | 22行 | **-57%** |
| new_extract_box | 51行 | 22行 | **-57%** |
| normalizer_box | 96行 | 74行 | **-23%** |
| emit_call_box | 56行 | 43行 | **-23%** |
| emit_method_box | 54行 | 41行 | **-24%** |
| emit_newbox_box | 54行 | 41行 | **-24%** |

---

## 🚀 推奨実装順序

### ✅ 今すぐ実施（Phase 1）
1. **Extract系統合**（2時間、-62行）
   - 最大の効果、最小のリスク
   - `stage1_int_args_extract_box.hako` 新設

2. **Normalizer共通化**（1.5時間、-22行）
   - 可読性大幅向上
   - テスト容易

3. **Emit系統合**（1.5時間、-27行）
   - 保守性向上

### ⚡ Phase完了後検討（Phase 2）
4. **RegexFlow最適化**（6-8時間）
   - 18-31%高速化の見込み
   - パフォーマンス重視

### 💭 将来検討（Phase 3）
5. **3層アーキテクチャ統合**（16-20時間）
   - 100-150行削減
   - 大規模リファクタリング

---

## 📋 実装チェックリスト

### Extract系統合
- [ ] `stage1_int_args_extract_box.hako` 新規作成（52行）
- [ ] `call_extract_box.hako` 更新（54→24行）
- [ ] `method_extract_box.hako` 更新（51→22行）
- [ ] `new_extract_box.hako` 更新（51→22行）
- [ ] テスト: 引数0/1/3個、不正JSON

### Normalizer共通化
- [ ] `normalizer_box.hako` 更新（96→74行）
  - [ ] `_normalize_int_array()` 追加
  - [ ] `_normalize_with_label_and_args()` 追加
  - [ ] 既存3メソッドを薄いラッパーに
- [ ] テスト: call/method/new全パターン

### Emit系統合
- [ ] `args_const_emit_box.hako` 新規作成（31行）
- [ ] `emit_call_box.hako` 更新（56→43行）
- [ ] `emit_method_box.hako` 更新（54→41行）
- [ ] `emit_newbox_box.hako` 更新（54→41行）
- [ ] テスト: v0/v1両バージョン

### 統合テスト
- [ ] `tools/smokes/v2/run.sh --profile quick` 全PASS
- [ ] セルフホストビルド確認
- [ ] パフォーマンス計測

---

## 🎯 Box-First設計スコア

### 現状評価: **78/100**

| 項目 | スコア | 評価 |
|------|-------|------|
| 責務分離 | 18/20 | ⭐ 良好 |
| 再利用性 | 14/20 | ⚠️ 重複多い |
| テスト性 | 16/20 | ⭐ 良好 |
| 保守性 | 15/20 | ⚠️ 改善余地 |
| パフォーマンス | 15/20 | ⚠️ 最適化必要 |

### Phase 1実装後: **85/100** 予想

- 再利用性: 14 → 18 (+4)
- 保守性: 15 → 18 (+3)

---

## 📚 関連ドキュメント

### 詳細分析
- **[pipeline_v2_boxification_analysis.md](./pipeline_v2_boxification_analysis.md)** (15KB)
  - 全体アーキテクチャ分析
  - パフォーマンス改善見込み
  - 3層統合提案

- **[pipeline_v2_duplication_details.md](./pipeline_v2_duplication_details.md)** (20KB)
  - 行単位の重複比較
  - 統合後コード例
  - 実装チェックリスト

---

## 💡 次のアクション

### 今すぐやること
1. このサマリーをユーザーに提示
2. Phase 1実施の承認取得
3. `stage1_int_args_extract_box.hako` 実装開始

### ユーザーへの質問
- Phase 1（6時間、111行削減）を今すぐ実施しますか？
- それとも、詳細分析レビュー後に判断しますか？
- Phase 2（RegexFlow最適化）の優先度はどうですか？

---

**結論**: 最小の労力（6時間）で最大の効果（111行削減、-3.9%）が得られるPhase 1の即座実施を強く推奨します。
