# ⚠️ SUPERSEDED: 戦略変更 Choice A'' (Macro-Only) に移行

**最新決定日**: 2025-10-08
**現在の戦略**: **Choice A'' (Macro-Only Approach)** ← [詳細](#strategic-pivot-choice-a-macro-only---current-decision)

---

# ~~Strategy C（段階的統合）採用決定~~ ← SUPERSEDED

**決定日**: 2025-10-08 (午前)
**決定者**: ユーザー判断（「全ての開発にかかわってきますにゃ」）
**分析**: ultrathink 長期コード品質分析（10年視点）
**ステータス**: ❌ **SUPERSEDED** - Choice A'' に変更（同日午後）

---

## 🎯 採用戦略

**Strategy C（段階的統合）**: 25-35人日（5-7週間）

```
Step 1: enum MVP実装（3-5人日）
  ├─ Option<T> 基本実装
  ├─ Result<T,E> 基本実装
  └─ 基本パターンマッチング

Step 2: Mini-VM実装 with enum MVP（10-15人日）
  ├─ 新規コードのみ Option<T>/Result<T,E> 使用
  ├─ 既存コードは最小限の修正
  └─ 技術的負債の新規追加を防ぐ

Step 3: セルフホスト達成（3-5人日）

Step 4: enum完全実装（Phase 20、10-15人日）
  ├─ @enum/@matchマクロ実装
  └─ 既存コード段階的リファクタリング
```

---

## 📊 3戦略比較（決定根拠）

### 短期視点（1ヶ月）
- **Strategy A（enum-first）**: 28-42人日、★★☆☆☆（遅い）
- **Strategy B（Mini-VM-first）**: 13-20人日、★★★★★（最速）← 当初提案
- **Strategy C（段階統合）**: 25-35人日、★★★☆☆（中間）

### 長期視点（10年）
- **Strategy A（enum-first）**: 品質★★★★★、技術的負債100 → 200
- **Strategy B（Mini-VM-first）**: 品質★☆☆☆☆、技術的負債100 → 800-1000
- **Strategy C（段階統合）**: 品質★★★★☆、技術的負債100 → 200-300 ← **採用**

### 総合評価（10年ROI）
```
Strategy B: 初期13-20人日節約、但し10年後 +50-100人日リファクタ
Strategy C: 初期+12-15人日追加、但し10年後 -50-100人日節約

→ Strategy C の方が10年累計で 60-115人日有利
```

---

## 🔍 技術的負債分析

### 現状（Phase 15.7時点）
**既存Mini-VM（2,379行、38ファイル）**:
- null チェック: 66箇所
- error コード（-1/-2/0）: 34箇所
- 品質スコア: 5/10（中〜高レベルの技術的負債）

### 10年累積モデル

#### Strategy B（Mini-VM-first）の場合
```
Phase 15.7:  100 debt points
Phase 20:    200 points（+新規nullチェック50、+新規errorコード50）
Phase 25:    400 points（複雑性の複利増加）
10年後:      800-1000 points

リファクタリングコスト: 50-100人日（遅延するほど増大）
```

#### Strategy C（段階統合）の場合
```
Phase 15.7:  100 debt points
Step 1:      +10 points（enum MVP実装）
Step 2:      +20 points（新規null/error禁止により50%削減）
Step 3:      +10 points（統合作業）
Step 4:      -60 points（段階的リファクタ）
10年後:      200-300 points

技術的負債削減: 70%（800-1000 → 200-300）
```

---

## 💡 決定の転機

### ユーザーの重要発言

> **「hakoruneセルフホスティング　コードは　綺麗にするのとても大切とおもいますにゃ　一番大本のrust vmからの立上げで　何かあったとき　ここからビルドする事も想定しますにゃ　全ての開発にかかわってきますにゃ」**

### この発言の意味

1. **Bootstrap Chainの信頼性が最優先**
   - Rust VM → Hakorune Selfhost Compiler → MIR JSON → VM実行
   - 10年以上メンテナンスする可能性

2. **技術的負債の複利的増加を回避**
   - 早期の悪習慣は後の全コードに伝播
   - "全ての開発にかかわってくる"

3. **短期的スピードより長期的品質**
   - 1ヶ月の差（13-20 vs 25-35人日）は許容範囲
   - 10年で50-100人日節約の方が重要

---

## 🚀 次のアクション

### 最優先タスク: Step 1（enum MVP実装）

**期間**: 3-5人日

**成果物**:
- `apps/lib/boxes/option.hako` - Option<T> 基本実装
- `apps/lib/boxes/result.hako` - Result<T,E> 基本実装
- `apps/tests/test_option_basic.hako` - テストスイート
- `apps/tests/test_result_basic.hako` - テストスイート
- `docs/guides/option-result-usage.md` - 使用ガイド

**成功基準**:
- [ ] Option<T> 基本操作すべて動作
- [ ] Result<T,E> 基本操作すべて動作
- [ ] スモークテスト PASS
- [ ] Mini-VMコードで使用可能な状態

---

## 📋 関連ドキュメント

- **実行計画**: [mini_vm_migration_plan.md](mini_vm_migration_plan.md)
- **進捗記録**: [mini_vm_progress.md](mini_vm_progress.md)
- **失敗記録**: [mini_vm_lessons.md](mini_vm_lessons.md)
- **VariantBox設計**: [docs/development/roadmap/phases/phase-20-variant-box/](../../roadmap/phases/phase-20-variant-box/)

---

**~~結論~~**: ~~Strategy C（段階的統合）により、短期的コスト（+12-15人日）を払って長期的品質（10年で-50-100人日）を獲得する戦略を採用。~~ ← **SUPERSEDED by Choice A''**

**Bootstrap Chainの信頼性 = プロジェクトの10年生存率**

---

## 🔄 Strategic Pivot: Choice A'' (Macro-Only) - CURRENT DECISION

### 📅 背景・転機

**日時**: 2025-10-08（Strategy C決定の数時間後）

**ユーザーの重要質問**:
> **「じゃあ　ultrathinkで計画修正　enum match だけ実装　これでいこう」**
> **「ガチガチに作ってきたからセルフホスティングもガチガチ大作戦だにゃ」**

**問題意識**:
- **「enum　なしで　セルフホスティング　めざすの？　綺麗にできるにゃ？」**
- Strategy C では enum MVP（基本実装）でスタートするが、@match マクロは Phase 20 以降
- セルフホストコードで pattern matching なしは「中途半端（half-baked）」では？

---

### 🎯 Choice A'' とは何か

**正式名称**: Choice A'' (Macro-Only Approach)

**核心原則**:
```
@enum/@match マクロ【のみ】実装
VariantBox Core は【実装しない】
→ 既存の Option/Result を _tag フィールドで代用
→ パターンマッチングの品質を半分の時間で達成
```

**実装内容**:
1. **@enum マクロ**
   - 入力: `@enum Result { Ok(value) Err(error) }`
   - 出力: コンストラクタ関数 + ヘルパー関数の自動生成
   - 内部表現: 既存の Option/Result Box + `_tag` フィールド

2. **@match マクロ**
   - 入力: `@match result { Ok(v) => ... Err(e) => ... }`
   - 出力: if-else チェーンへの脱糖
   - パターン: タグベース分岐のみ（基本パターンのみ）

3. **Out of Scope（Phase 20 以降に延期）**
   - ❌ VariantBox Core 実装
   - ❌ EnumSchemaBox
   - ❌ SymbolBox（タグの最適化）
   - ❌ コンパイル時 exhaustiveness checking
   - ❌ 高度なパターン（ガード、リテラル、ネスト）

---

### 📊 Choice A'' vs 他戦略の比較

| 項目 | Choice A<br>(Full enum) | Choice A''<br>**(Macro-Only)** | Strategy B<br>(Mini-VM first) | Strategy C<br>(Phased) |
|------|------------------------|-------------------------------|------------------------------|----------------------|
| **実装期間** | 28-42人日 | **9-14人日** ⭐ | 13-20人日 | 25-35人日 |
| **Pattern Matching達成** | Week 4-6 | **Week 2-3** ⭐ | ❌ Never | Week 5-7 |
| **品質（10年後）** | ★★★★★ | ★★★★☆ ⭐ | ★☆☆☆☆ | ★★★★☆ |
| **セルフホストコード品質** | 最高 | **高** ⭐ | 最低 | 中〜高 |
| **技術的負債** | 最小 | **小** ⭐ | 最大 | 中 |
| **中途半端リスク** | なし | **なし** ⭐ | 高 | 中（MVP期間） |

**⭐ = Choice A'' の優位点**

---

### 💡 Choice A'' が最適な理由

#### 1️⃣ 「中途半端」問題の完全回避

**最悪のケース**: VariantBox あり + @match なし
- 手動で `if box.is_tag("Ok")` を毎回書く
- セルフホストコード全体に伝播
- 「enum があるのにパターンマッチングできない」← 最も中途半端

**Choice A'' の解決**:
- VariantBox Core を実装しない
- @enum/@match を同時に提供
- 「完全なパターンマッチング体験」を最短で実現

#### 2️⃣ 時間対効果の最適化

```
Choice A  (Full): 28-42日 → Pattern Matching ⭕
Choice A'' (Macro): 9-14日 → Pattern Matching ⭕  ← 同じ結果、半分の時間！

時間削減: 14-28日（50-67%短縮）
品質差: ほぼなし（セルフホスト用途では同等）
```

**削減内訳**:
- VariantBox Core 実装: -10日
- EnumSchemaBox: -3日
- SymbolBox: -5日
- 高度パターン: -7日
合計: **-25日**

**残す部分**:
- @enum マクロ: 4-5日
- @match マクロ: 5-9日
合計: **9-14日**

#### 3️⃣ セルフホスト「ガチガチ大作戦」の実現

**ユーザーの意図**:
> **「ガチガチに作ってきたからセルフホスティングもガチガチ大作戦だにゃ」**

**Choice A'' の応答**:
- ✅ セルフホストコードは **100% @match** で書ける
- ✅ エラー処理は **全て @match** で統一
- ✅ null チェックは **@match Option** で置き換え
- ✅ 「中途半端」な状態は **1日たりとも存在しない**

**Strategy C との違い**:
- Strategy C: enum MVP期間（3-5日）は @match なし ← 中途半端
- Choice A'': @enum/@match 同時実装 ← 完全

#### 4️⃣ Bootstrap Chain の品質保証

**10年視点での評価**:

| 戦略 | セルフホストコード品質 | パターンマッチング | 保守性 |
|------|---------------------|-----------------|-------|
| Choice A (Full) | ★★★★★ | Full (Week 4-6) | 最高 |
| **Choice A'' (Macro)** | **★★★★☆** | **Full (Week 2-3)** | **高** |
| Strategy C | ★★★☆☆ | MVP → Full | 中〜高 |
| Strategy B | ★☆☆☆☆ | ❌ | 低 |

**Choice A'' の10年後**:
- セルフホストコードは全て @match で統一
- 将来の VariantBox Core 追加は **透過的**（マクロの内部実装を変えるだけ）
- 技術的負債: 小（@match の脱糖コードのみ、100% 予測可能）

---

### 🏗️ Choice A'' 実装スコープ（詳細）

#### Week 1: @enum マクロ実装（4-5日）

**Day 1-2**: パーサー拡張（Rust側）
- `@enum` 構文解析
- AST ノード: `EnumDeclaration`
- 既存の `@derive` 実装を参考

**Day 3-4**: マクロエンジン（Hakorune側）
```hakorune
// 入力
@enum Result {
    Ok(value)
    Err(error)
}

// 出力（自動生成）
static box Result {
    Ok(v) {
        local r = new ResultBox()
        r._tag = "Ok"
        r._value = v
        return r
    }

    Err(e) {
        local r = new ResultBox()
        r._tag = "Err"
        r._error = e
        return r
    }

    is_ok(r) { return r._tag == "Ok" }
    is_err(r) { return r._tag == "Err" }
    unwrap_ok(r) {
        if r._tag != "Ok" { panic("unwrap_ok on Err") }
        return r._value
    }
    unwrap_err(r) {
        if r._tag != "Err" { panic("unwrap_err on Ok") }
        return r._error
    }
}
```

**Day 5**: テスト・スモークテスト
- 10パターンの @enum テスト
- Option/Result の @enum 版実装

**成果物**:
- `apps/macros/enum/enum_macro.hako` (100-150行)
- `apps/lib/boxes/option_enum.hako` (使用例)
- `apps/lib/boxes/result_enum.hako` (使用例)

---

#### Week 2-3: @match マクロ実装（5-9日）

**Day 1-3**: パーサー拡張（Rust側）
- `@match` 構文解析
- パターン構文: `Tag(bindings)` のみ
- AST ノード: `MatchExpression`

**Day 4-7**: マクロエンジン（Hakorune側）
```hakorune
// 入力
@match result {
    Ok(value) => {
        console.log("Success: " + value)
        return value
    }
    Err(error) => {
        console.error("Error: " + error)
        return null
    }
}

// 出力（脱糖）
local __match_result = result
if __match_result._tag == "Ok" {
    local value = __match_result._value
    console.log("Success: " + value)
    return value
} else if __match_result._tag == "Err" {
    local error = __match_result._error
    console.error("Error: " + error)
    return null
} else {
    panic("Non-exhaustive match: unknown tag " + __match_result._tag)
}
```

**Day 8-9**: 統合テスト
- 15パターンの @match テスト
- Option/Result の実用例
- Mini-VM エラーハンドリングへの適用

**成果物**:
- `apps/macros/match/match_macro.hako` (150-200行)
- `apps/tests/match_patterns.hako` (テストスイート)
- Mini-VM の 3-5ファイルを @match で書き換え（実証）

---

### ✅ 成功基準（Choice A''）

#### Phase 19 完了 = 以下すべて満たす

1. **@enum マクロ動作**
   - [ ] 10/10 @enum テスト PASS
   - [ ] Option/Result を @enum で定義可能
   - [ ] コンストラクタ自動生成動作

2. **@match マクロ動作**
   - [ ] 15/15 @match テスト PASS
   - [ ] if-else への正しい脱糖
   - [ ] 非網羅パターンで panic 動作

3. **セルフホストコード適用**
   - [ ] Mini-VM の 3-5ファイルを @match で書き換え
   - [ ] null チェック → @match Option に置き換え
   - [ ] エラーコード → @match Result に置き換え

4. **スモークテスト**
   - [ ] Quick profile 全 PASS
   - [ ] Integration profile 全 PASS
   - [ ] 性能劣化なし（脱糖コードは効率的）

5. **ドキュメント**
   - [ ] @enum/@match 使用ガイド
   - [ ] マイグレーションガイド（null → Option, -1/-2 → Result）
   - [ ] Phase 20 への移行計画（VariantBox Core追加）

---

### 🚨 リスク管理（Choice A''）

#### 🔴 クリティカル

1. **@構文パーサー拡張の複雑さ**
   - 影響: 高（Rust側のパーサー拡張が難航）
   - 確率: 中（Phase 16 で @derive 実装経験あり）
   - 対策:
     - Phase 16 の @derive 実装を詳細に分析
     - シンプルな構文から段階実装
     - 必要なら構文を簡略化（`@enum` → `enum!` マクロ等）

2. **マクロ展開の複雑性**
   - 影響: 高（脱糖コードのバグ）
   - 確率: 中（200-350行のマクロコード）
   - 対策:
     - 詳細なテストスイート（25パターン）
     - 既存の loop_normalize_macro.nyash（393行）を参考
     - ChatGPT Pro によるコードレビュー

#### 🟡 中程度

3. **既存 Option/Result との互換性**
   - 影響: 中（移行期の混在）
   - 確率: 低（別名で実装可能）
   - 対策:
     - `option_enum.hako` / `result_enum.hako` として別実装
     - 段階移行計画（3-5ファイルずつ）
     - 互換レイヤー実装（必要なら）

4. **性能劣化**
   - 影響: 中（脱糖コードの効率）
   - 確率: 低（if-else は VM 最適化済み）
   - 対策:
     - ベンチマーク測定
     - 必要なら脱糖コードを最適化

#### 🟢 軽微

5. **網羅性チェックなし**
   - 影響: 低（実行時エラーで検出）
   - 確率: 確定（静的チェックは Phase 25）
   - 対策:
     - ドキュメントで明記
     - テストで全パターンをカバー
     - Phase 25 で静的チェック追加

---

### 🔄 Rollback Plan（万が一の場合）

#### Phase 19 失敗時の対応

**判断基準**:
- Week 3 終了時点で 25/25 テストの 50% 以上が FAIL
- 性能が 2倍以上劣化
- パーサー拡張が技術的に不可能と判明

**Rollback 先**:
1. **Option A: Strategy C に戻る**
   - enum MVP（基本実装のみ）で進行
   - @match は Phase 20 に延期
   - 期間: +2週間（合計 5-7週間）

2. **Option B: Strategy B に変更**
   - enum を完全に諦める
   - Mini-VM 実装優先
   - 品質は妥協（技術的負債増加）

**推奨**: Option A（Strategy C への Rollback）
- 理由: 「中途半端」は避けるべきだが、完全失敗より「基本enum」のほうがマシ

---

### 📚 参考資料（Choice A''）

#### 既存実装・設計書
- **Phase 20 VariantBox設計**: `docs/development/roadmap/phases/phase-20-variant-box/DESIGN.md`
- **Result Box設計**: `docs/development/roadmap/phases/phase-20-variant-box/RESULT_BOX_COMPLETE_DESIGN.md`
- **Phase 16 マクロ実装**: `docs/development/roadmap/phases/phase-16-macro-revolution/README.md`

#### 既存マクロ実装（参考コード）
- `apps/macros/loop_normalize_macro.nyash` (393行) - 複雑な脱糖の実例
- `apps/macros/if_match_normalize_macro.nyash` (404行) - パターンマッチング的な構文
- `src/macro/pattern.rs` (252行) - Rust側のパターンマッチング実装

#### 実装済み Option/Result
- `apps/lib/boxes/option.hako` - 既存実装（commit: e441b2ba）
- `apps/lib/boxes/result.hako` - 既存実装（commit: e441b2ba）
- `apps/selfhost/vm/boxes/result_box.hako` (34行) - Mini-VM 用実装

---

### 🎯 次のアクション（Choice A'' 実装開始）

#### 準備フェーズ（Day 0、半日）

1. **環境確認**
   - [ ] Hakorune ビルド確認
   - [ ] スモークテスト実行（baseline測定）
   - [ ] Phase 16 マクロシステム動作確認

2. **設計レビュー**
   - [ ] Phase 20 VariantBox 設計精読
   - [ ] 既存 @derive 実装精読（`src/macro/`）
   - [ ] loop_normalize_macro.nyash 精読（脱糖パターン学習）

3. **タスク準備**
   - [ ] `docs/development/roadmap/phases/phase-19-enum-match/README.md` 作成
   - [ ] Week 1 タスクリスト作成
   - [ ] 進捗記録ファイル準備

#### Week 1 開始（Day 1）

**タスク**: @enum パーサー拡張
- [ ] `src/parser/mod.rs` 分析
- [ ] `@enum` 構文設計
- [ ] AST ノード `EnumDeclaration` 追加
- [ ] 最小テストケース作成

---

## 📊 Strategic Decision Summary

### 決定の変遷

```
2025-10-08 午前: Strategy C 採用
    ↓
    ユーザー質問: 「enum なしでセルフホスティング　めざすの？」
    ↓
2025-10-08 午後: Choice A'' (Macro-Only) に変更
```

### 最終決定

**採用戦略**: **Choice A'' (Macro-Only Approach)**

**理由**:
1. 「中途半端」の完全回避
2. パターンマッチング到達時間: 半分（2-3週 vs 5-7週）
3. セルフホストコード品質: 高（100% @match 統一）
4. Bootstrap Chain 信頼性: 高維持
5. 技術的負債: 小（予測可能）

**トレードオフ**:
- ✅ 得るもの: 2-3週間の時間短縮、完全なパターンマッチング体験
- ❌ 失うもの: VariantBox Core の将来的な柔軟性（Phase 20 で追加可能）

**ユーザーの意図との整合性**:
> **「ガチガチに作ってきたからセルフホスティングもガチガチ大作戦だにゃ」**

Choice A'' は「ガチガチ」を実現:
- ✅ セルフホストコードは 100% @match で統一
- ✅ エラー処理は全て型安全
- ✅ 「中途半端」な状態は 1日たりとも存在しない

---

**結論（最新）**: **Choice A'' (Macro-Only)** により、パターンマッチングを最短（9-14日）で実現し、セルフホストコードの「ガチガチ大作戦」品質を保証する戦略を採用。

**Bootstrap Chain の信頼性 = パターンマッチングの完全性**
