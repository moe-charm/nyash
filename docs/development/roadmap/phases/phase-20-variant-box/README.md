# Phase 20.X: VariantBox + enum/match実装

**ステータス**: PLANNED（Phase 15.7セルフホスト完了後に開始）
**期間**: 18-27人日（4-6週間、1人体制）
**前提条件**: Phase 15.7完了、マクロシステム安定化（Phase 16暫定版）

---

## 🎯 目的

Hakoruneにenum的な表現を追加（MIR16維持、Everything-is-Box原則を守る）

**設計元**: ChatGPT Pro提案（2025-10-08）
**分析完了**: ultrathink 4タスク並列分析（整合性・代替案・リスク・影響）

---

## 📋 実装内容

### Phase 20.6: VariantBox Core（3-5人日）

**目標**: enum的な表現の最小実装（マクロなし）

#### タスク
- [ ] **VariantBox実装**（2日）
  ```hakorune
  box VariantBox {
      tag: StringBox           // "Some", "None", "Ok", "Err" 等
      fields: ArrayBox         // 位置引数

      birth(tag_str) {
          me.tag = new StringBox(tag_str)
          me.fields = new ArrayBox()
      }

      get_tag() { return me.tag }
      get_fields() { return me.fields }
      is_tag(name) { return me.tag.equals(name) }
      field(i) {
          if i < 0 || i >= me.fields.len() {
              panic("VariantBox.field: index out of bounds")
          }
          return me.fields.get(i)
      }
  }
  ```

- [ ] **EnumSchemaBox実装**（1日）
  ```hakorune
  static box EnumSchemaBox {
      name: StringBox              // "Result", "Option" 等
      variants: ArrayBox           // ["Ok", "Err"]

      register_variant(tag, arity) { ... }
      validate_variant(variant_box) { ... }
  }
  ```

- [ ] **手動コンストラクタテスト**（1日）
  ```hakorune
  // apps/tests/variant_basic.hako
  local some = new VariantBox("Some")
  some.fields.push(42)

  if some.is_tag("Some") {
      print(some.field(0))  // → 42
  }
  ```

**成果物**:
- `apps/lib/boxes/variant_box.hako` (50-80行)
- `apps/lib/boxes/enum_schema_box.hako` (40-60行)
- `apps/tests/variant_basic.hako` (30-50行)

---

### Phase 20.7: @enumマクロ（5-7人日）

**目標**: @enum構文でコンストラクタ関数を自動生成

#### タスク
- [ ] **パーサー@enum対応**（2日、Rust側）
  - `src/parser/mod.rs`: @enum構文追加（100-150行）
  - 新AST型: `ASTNode::EnumDeclaration`

- [ ] **@enumマクロ実装**（3日、Hakorune側）
  ```hakorune
  // ユーザーコード
  @enum Result {
      Ok(value)
      Err(error)
  }

  // 自動生成（脱糖先）:
  static box Result {
      Ok(value) {
          local v = new VariantBox("Ok")
          v.fields.push(value)
          return v
      }

      Err(error) {
          local v = new VariantBox("Err")
          v.fields.push(error)
          return v
      }

      is_ok(r) { return r.is_tag("Ok") }
      as_ok(r) { return r.field(0) }

      is_err(r) { return r.is_tag("Err") }
      as_err(r) { return r.field(0) }
  }
  ```

- [ ] **スモークテスト追加**（1日）
  - `tools/smokes/v2/profiles/quick/core/enum_basic.sh`

**成果物**:
- `src/parser/mod.rs` 変更（100-150行）
- `apps/macros/enum/enum_macro.hako` (100-150行)
- `apps/macros/enum/README.md`

---

### Phase 20.8: @matchマクロ（7-10人日）

**目標**: パターンマッチング構文（if/else脱糖）

#### タスク
- [ ] **パーサー@match対応**（3日、Rust側）
  - `src/parser/mod.rs`: @match構文追加（80-120行）
  - 新AST型: `ASTNode::MatchExpression`

- [ ] **@matchマクロ実装**（4日、Hakorune側）
  ```hakorune
  // ユーザーコード
  @match result {
      Ok(v) => console.log(v)
      Err(e) => console.error(e)
  }

  // 脱糖先:
  if result.is_tag("Ok") {
      local v = result.field(0)
      console.log(v)
  } else if result.is_tag("Err") {
      local e = result.field(0)
      console.error(e)
  } else {
      panic("Non-exhaustive match")
  }
  ```

- [ ] **網羅性チェック（基本版）**（2日）
  - 実行時エラー（`panic("Non-exhaustive match")`）
  - 静的チェックはPhase 25（型パス）で追加

**成果物**:
- `src/parser/mod.rs` 変更（80-120行）
- `apps/macros/match/match_macro.hako` (150-200行)
- `apps/macros/match/README.md`

---

### Phase 20.9: Option/Result enum化（3-5人日）

**目標**: 既存Option/ResultBoxをVariantBox版に段階移行

#### タスク
- [ ] **VariantBox版Option実装**（1日）
  ```hakorune
  @enum Option {
      Some(value)
      None
  }
  ```

- [ ] **VariantBox版Result実装**（1日）
  ```hakorune
  @enum Result {
      Ok(value)
      Err(error)
  }
  ```

- [ ] **段階移行計画実施**（2日）
  - 新規コードのみVariantBox版使用
  - 既存ResultBox（`selfhost/vm/boxes/result_box.hako`）は互換維持
  - 3-5ファイル試験移行

**成果物**:
- `apps/lib/boxes/option.hako` (60-80行)
- `apps/lib/boxes/result.hako` (60-80行)
- 移行ガイド: `docs/guides/option-result-migration.md`

---

## 📊 マイルストーン

| Phase | 成果物 | 成功基準 | リスク |
|-------|--------|---------|--------|
| **20.6** | VariantBox Core | 手動コンストラクタで動作 | 🟢 低 |
| **20.7** | @enumマクロ | コンストラクタ自動生成 | 🟡 中（パーサー拡張） |
| **20.8** | @matchマクロ | パターンマッチ動作 | 🟡 中（複雑な脱糖） |
| **20.9** | Option/Result | 既存コード一部移行 | 🟢 低 |

**合計**: 18-27人日（4-6週間）

---

## ⚠️ 前提条件

### 必須
- ✅ Phase 15.7完了（セルフホスト達成）
- ✅ StringBox/ArrayBox/MapBox安定稼働
- ✅ マクロシステム基本機能（Phase 16暫定版で可）

### 推奨
- StringBox比較の効率化（SymbolBox実装は後回し可能）
- パーサー拡張の経験蓄積

---

## 🚨 既知のリスク

### 🔴 クリティカル
1. **SymbolBox未実装**
   - 対策: Phase 20.6-20.9は**StringBoxタグ版**で実装
   - SymbolBox（インターン機構）はPhase 25で追加可能

2. **@構文パーサー拡張**
   - 現状: @構文の実装例なし
   - 対策: Phase 16で@derive実装時に基盤整備

### 🟡 中程度
3. **網羅性チェック**
   - Phase 20では実行時エラーのみ
   - 静的チェックはPhase 25（型パス）で追加

4. **既存コード移行コスト**
   - 影響範囲: ResultBox 5箇所のみ（現状）
   - 対策: 段階的移行、互換API維持

---

## 📚 参考資料

### Phase 20設計書
- **[DESIGN.md](./DESIGN.md)** - VariantBox完全設計（主要文書）
- **[RESULT_BOX_COMPLETE_DESIGN.md](./RESULT_BOX_COMPLETE_DESIGN.md)** - Result<T,E> Box完全設計（2025-10-08新規）
- **[RESULT_BOX_SUMMARY.md](./RESULT_BOX_SUMMARY.md)** - Result Box 1分サマリー
- **[RESULT_BOX_MIGRATION_PLAN.md](./RESULT_BOX_MIGRATION_PLAN.md)** - Result Box段階移行計画
- **[result_box_v2_reference.hako](./result_box_v2_reference.hako)** - Result Box参照実装（467行）

### 実装計画書（2025-10-08新規）
- **[ENUM_MATCH_PROJECT_PLAN.md](./ENUM_MATCH_PROJECT_PLAN.md)** - @enum/@match完全プロジェクト計画（12-17日）
  - 日次タスク詳細、リスク分析、成功基準、ロールバック計画
- **[ENUM_MATCH_QUICK_START.md](./ENUM_MATCH_QUICK_START.md)** - クイックスタートガイド（要約版）
  - 3週間スケジュール、重要リスク、日次チェックリスト

### 設計提案元
- ChatGPT Pro提案（2025-10-08）
- VariantBox設計完全分析（ultrathink 4タスク）

### 関連Phase
- Phase 16: マクロ革命（@derive実装）
- Phase 20.0-20.5: マクロフル機能（基盤整備）
- Phase 25: 型パス（静的網羅性チェック）

### 実装参考
- `apps/macros/loop_normalize_macro.nyash` (393行)
- `apps/macros/if_match_normalize_macro.nyash` (404行)
- `src/macro/pattern.rs` (252行)
- `selfhost/vm/boxes/result_box.hako` (34行、既存実装)

---

## 🎯 Phase 20内の優先順位

```
Phase 20.0-20.3（マクロ基盤＋derive＋test）← 🔴 P1（最優先）
    ↓
Phase 20.6-20.9（VariantBox）← 🟢 P2（推奨）
    ↓
Phase 20.4-20.5（Pattern/Quote）← 🟡 P3（必要時）
    ↓
Python統合（Plan B）← 🔵 P4（長期計画）
```

**理由**:
- P1: セルフホストコードの生産性向上に直結
- P2: 型安全性向上（オプショナルだが効果大）
- P3: 複雑マクロ用（必要になってから実装）
- P4: Phase 25以降でも可

---

## 📝 次のアクション

### Phase 15.7完了後すぐ
1. Phase 20.0開始（JsonAstBox基盤）
2. Phase 20.6設計レビュー
3. VariantBox Core実装開始

### Phase 20.6完了後
1. @enumマクロ設計
2. パーサー@構文拡張計画
3. Phase 20.7実装開始

---

**作成日**: 2025-10-08
**最終更新**: 2025-10-08
**ステータス**: 計画済み、実装待ち
