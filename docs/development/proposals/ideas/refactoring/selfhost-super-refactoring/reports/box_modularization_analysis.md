# 箱化・モジュール化現状分析レポート

**分析対象**: apps/selfhost-compiler (mini-vmは存在せず)
**分析日時**: 2025-10-04
**目的**: セルフホストコンパイラの箱化・モジュール化の品質評価と改善提案

---

## 📊 1. 箱化率（Box-ification Rate）

### ファイル統計

| 指標 | 数値 |
|-----|------|
| **総ファイル数** | 57 |
| **.hako ファイル** | 43 (75.4%) |
| **.nyash ファイル** | 14 (24.6%) |

```
箱化率: 75.4% (.hako形式)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
███████████████████████████████████████████████░░░░░░░░░░░░░░ 75.4%
```

### 未箱化ファイル (.nyash)

1. `apps/selfhost-compiler/parser/lexer.nyash` (レガシー版)
2. `apps/selfhost-compiler/parser/parser.nyash` (レガシー版)
3. `apps/selfhost-compiler/parser/ast.nyash` (レガシー版)
4. `apps/selfhost-compiler/emitter/json_v0.nyash` (レガシー版)
5. `apps/selfhost-compiler/boxes/debug_box.nyash` (並行版)
6. `apps/selfhost-compiler/boxes/mir_emitter_box.nyash` (並行版)
7. `apps/selfhost-compiler/builder/mod.nyash` (レガシー版)
8. `apps/selfhost-compiler/builder/ssa/local.nyash` (レガシー版・547行の大型モジュール)
9. `apps/selfhost-compiler/builder/ssa/loopssa.nyash` (レガシー版)
10. `apps/selfhost-compiler/builder/rewrite/known.nyash` (レガシー版)
11. `apps/selfhost-compiler/builder/rewrite/special.nyash` (レガシー版)
12. `apps/selfhost-compiler/mir/builder.nyash` (レガシー版)
13. `apps/selfhost-compiler/mir/optimizer.nyash` (レガシー版)
14. `apps/selfhost-compiler/interfaces.nyash` (レガシー版)

**評価**: ほとんどの.nyashファイルには対応する.hakoファイルが存在しており、レガシーコードとして保持されている。これは適切な移行戦略。

---

## 📦 2. Box定義分析

### Box定義総数

- **通常Box**: 6個 (birth構文を使う動的インスタンス)
- **Static Box**: 70個以上 (シングルトン・ユーティリティ)
- **Stub Box**: 20個 (テスト用エントリーポイント)

### 主要Box一覧

#### 🔹 パーサー系 (Parser Module)
- `ParserBox` (921行) - **⚠️ 肥大化警告**
  - 59メソッド - **🔴 複雑すぎる！**
  - 責務: レキシング・パース・AST生成・using抽出・JSON出力
  - 問題: 単一責任の原則違反（複数の責務を持つ）

- `Lexer` (static)
- `Parser` (static)
- `AST` (static)

#### 🔹 エミッター系 (Emitter Module)
- `EmitterBox` (static)
- `MirEmitterBox` (static, 256行)
- `JsonV0Emitter` (static)
- `JsonProgramBox` (static, 520行)

#### 🔹 Pipeline v2系 (24ファイル・モジュール化良好)
Pipeline v2は良好にモジュール化されている：

| Box名 | 行数 | 責務 | 評価 |
|-------|------|------|------|
| `ExecutionPipelineBox` | 37 | パイプライン実行制御 | ✅ 適切 |
| `BackendBox` | 11 | バックエンド抽象化 | ✅ 適切 |
| `MirBuilderBox` | 34 | MIR構築 | ✅ 適切 |
| `LocalSSABox` | 105 | SSA変換・Copy挿入 | ✅ 適切 |
| `NormalizerBox` | 116 | 値正規化 | ✅ 適切 |
| `MapHelpersBox` | 65 | Map操作ユーティリティ | ✅ 適切 |
| `ReadOnlyMapView` | 30 | 読み取り専用Map | ✅ 適切 |
| `RegexFlow` | 103 | 正規表現風パターン | ✅ 適切 |

**Emit系Box** (MIR命令生成):
- `EmitReturnBox` (21行) ✅
- `EmitBinopBox` (33行) ✅
- `EmitCompareBox` (66行) ✅
- `EmitCallBox` (75行) ✅
- `EmitMethodBox` (56行) ✅
- `EmitNewBoxBox` (49行) ✅

**Extract系Box** (AST抽出):
- `CompareExtractBox` (126行) ⚠️ やや大きめ
- `CallExtractBox` (54行) ✅
- `MethodExtractBox` (51行) ✅
- `NewExtractBox` (51行) ✅

**Flow系Box** (制御フロー):
- `Stage1ExtractFlow` (206行) ⚠️ 大きめ
- `FlowEntryBox` (20行) ✅
- `PipelineV2` (382行・flow定義) ⚠️ 大きめ

#### 🔹 Builder系 (SSA/Rewrite)
- `MirBuilder` (static)
- `LocalSSA` (static, 547行 .nyash版) - **⚠️ 肥大化**
- `LoopSSA` (static)
- `CondInserter` (static, 117行)
- `RewriteKnown` (static)
- `RewriteSpecial` (static)
- `Optimizer` (static)

#### 🔹 デバッグ系
- `DebugBox` (36行) ✅

---

## 🎯 3. モジュール設計評価

### 3.1 モジュール間の結合度

#### ✅ 良好な例: Pipeline v2

Pipeline v2は**疎結合**で**高凝集**の優れた設計：

```
ExecutionPipelineBox (Entry Point)
  ↓ using
  ├─ ParserBoxMod.ParserBox
  ├─ EmitterBoxMod.EmitterBox
  └─ BackendBoxMod.BackendBox

PipelineV2 (Orchestrator)
  ↓ using (17モジュール)
  ├─ Stage1ExtractFlow
  ├─ EmitReturnBox, EmitBinopBox, EmitCompareBox (Emit系)
  ├─ CompareExtractBox, CallExtractBox (Extract系)
  ├─ LocalSSA, NormalizerBox (Utility)
  └─ MirJsonV1Adapter (External)
```

**評価**:
- ✅ 明確な責務分離
- ✅ using systemによる依存関係の可視化
- ✅ 小さなBoxを組み合わせる設計
- ✅ テスト可能性（各BoxにStubが存在）

#### ⚠️ 問題のある例: ParserBox

`ParserBox` (921行) は**密結合**で**低凝集**:

```
ParserBox
  ├─ レキシング (is_digit, is_alpha, is_space)
  ├─ パース (parse_program2)
  ├─ using抽出 (extract_usings, add_using)
  ├─ 文字列操作 (trim, index_of, starts_with)
  ├─ JSON生成 (多数のヘルパー)
  └─ グローバル状態管理 (gpos, usings_json, stage3)
```

**問題点**:
- 🔴 単一責任の原則違反（5つ以上の責務）
- 🔴 59メソッド（10メソッド超過基準の約6倍）
- 🔴 テスト困難（モックが作りにくい）
- 🔴 再利用困難（必要な機能だけ取り出せない）

### 3.2 インターフェース明確性

**INTERFACES.md** が存在し、主要Boxのインターフェースが文書化されている ✅

主要インターフェース:
```
ParserBox:
  - stage3_enable(flag: i64) -> Void
  - extract_usings(src: String) -> Void
  - get_usings_json() -> String
  - parse_program2(src: String) -> String

EmitterBox:
  - emit_program(json: String, usings_json: String) -> String

ExecutionPipelineBox:
  - birth(name: String="vm") -> i64
  - run_source(src: String, stage3_flag: i64=0) -> i64

LocalSSABox:
  - ensure_after_phis_copy(insts: ArrayBox, src: i64, dst: i64) -> i64
  - add_copy(insts: ArrayBox, dst: i64, src: i64) -> i64
```

**評価**:
- ✅ インターフェース文書化の努力
- ✅ Fail-Fast原則の明示
- ⚠️ すべてのBoxが文書化されていない（pipeline_v2の多くが未記載）

### 3.3 共通モジュールの抽出可能性

#### 🔍 重複検出結果

以下のユーティリティ関数が**複数ファイルに重複**:

| 関数名 | 出現回数 | 重複箇所 |
|--------|---------|---------|
| `trim()` | 2回 | ParserBox, JsonProgramBox |
| `index_of()` | 5ファイル | ParserBox, JsonProgramBox, MirEmitterBox, RegexFlow, Stage1ExtractFlow |
| `_to_i64()` | 3回 | PipelineV2, NormalizerBox, (他) |
| `i2s()` | 2回 | ParserBox, (他) |
| `skip_ws()` | 2回 | (詳細不明) |

#### 📦 抽出可能な共通モジュール

**提案1: StringUtilsBox**
```hakorune
static box StringUtilsBox {
  trim(s: String) -> String
  index_of(src: String, start: i64, pattern: String) -> i64
  starts_with(src: String, pos: i64, prefix: String) -> bool
  last_index_of(src: String, pattern: String) -> i64
}
```

**提案2: NumberUtilsBox**
```hakorune
static box NumberUtilsBox {
  to_i64(v: Any) -> i64  // 文字列/数値→i64変換
  i2s(v: i64) -> String  // i64→文字列変換
  parse_int_at(s: String, idx: i64) -> i64?
}
```

**提案3: JsonUtilsBox**
```hakorune
static box JsonUtilsBox {
  extract_value(json: String, key: String) -> String?
  extract_string_value(json: String, key: String, default: String) -> String
  quote(s: String) -> String  // "value" 形式に
  escape_json(s: String) -> String
}
```

---

## 🐛 4. コード品質分析

### 4.1 長すぎるファイル（100行超過）

| ファイル | 行数 | 評価 | 対策 |
|---------|------|------|------|
| `ParserBox` | 921行 | 🔴 緊急 | 分割必須 |
| `LocalSSA` (.nyash) | 547行 | 🔴 緊急 | .hako化 + 分割 |
| `JsonProgramBox` | 520行 | 🔴 要改善 | 分割検討 |
| `PipelineV2` | 382行 | 🟡 許容範囲 | flow定義のため許容 |
| `MirEmitterBox` | 256行 | 🟡 監視対象 | - |
| `Stage1ExtractFlow` | 206行 | 🟡 監視対象 | - |
| `CompareExtractBox` | 126行 | ✅ 問題なし | - |
| `CondInserter` | 117行 | ✅ 問題なし | - |
| `NormalizerBox` | 116行 | ✅ 問題なし | - |

### 4.2 複雑すぎるBox（メソッド10個超）

| Box | メソッド数 | 評価 |
|-----|-----------|------|
| `ParserBox` | 59 | 🔴 緊急対応 |
| その他 | <10 | ✅ 問題なし |

**ParserBoxの問題詳細**:
- 59メソッド = 基準の約6倍
- 複雑度が高すぎてメンテナンス困難
- テストカバレッジが低い可能性

### 4.3 birth構文の使用状況

#### birth使用率
- birth定義のあるBox: 5ファイル
- 総Boxファイル数: 43
- **birth使用率: 11.6%**

#### birth使用Box
1. `ParserBox` ✅
2. `DebugBox` ✅
3. `ExecutionPipelineBox` ✅
4. `BackendBox` ✅
5. `MirBuilderBox` ✅

**評価**:
- ✅ 動的インスタンスが必要なBoxのみbirthを使用（適切）
- ✅ ほとんどがstatic box（シングルトンパターン）で適切

### 4.4 using systemの使用状況

#### using文の出現パターン
- **pipeline_v2/pipeline.hako**: 17個のusing（最多）
- **pipeline_v2/execution_pipeline_box.hako**: 5個
- **compiler.hako**: 6個

**評価**:
- ✅ using systemを積極的に活用
- ✅ 依存関係が明示的
- ✅ モジュール間の境界が明確

---

## 🏆 5. ベストプラクティス準拠度

### 5.1 Everything is Box 原則

| 評価項目 | スコア | 詳細 |
|---------|-------|------|
| Box化の徹底 | 90/100 | ほぼすべての機能がBox化 |
| Box責務の明確化 | 75/100 | Pipeline v2は優秀、ParserBoxは問題 |
| Box間の疎結合 | 85/100 | using systemで適切に分離 |

### 5.2 birth構文の使用

| 評価項目 | スコア | 詳細 |
|---------|-------|------|
| birth適用率 | 95/100 | 必要な箇所のみ使用（適切） |
| birth内の初期化 | 90/100 | シンプルな初期化に留める |

### 5.3 エラーハンドリングの統一性

**INTERFACES.md**にFail-Fast原則が明記 ✅

```
Fail-Fast: interfaces return explicit errors; no implicit fallbacks.
```

**実装例**:
```hakorune
// LocalSSABox - 明示的なエラー返却
add_copy(insts, dst, src) {
  if insts == null { return 1 }  // Fail-Fast
  if insts.push == null { return 1 }  // Fail-Fast
  insts.push({ op:"copy", dst: dst, src: src })
  return 0  // Success
}
```

**評価**:
- ✅ Fail-Fast原則が文書化
- ⚠️ 一部のBoxでまだ暗黙的なフォールバックが残存
- 🔴 ParserBox内のエラーハンドリングが不統一

---

## 📈 6. 箱化率グラフ（テキストベース）

### 全体の箱化状況
```
┌────────────────────────────────────────────────────────────────┐
│ 箱化進捗 (.hako vs .nyash)                                      │
├────────────────────────────────────────────────────────────────┤
│ .hako (箱化済み)  75.4% ███████████████████████████░░░░░░░     │
│ .nyash (未箱化)   24.6% ████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  │
└────────────────────────────────────────────────────────────────┘
```

### モジュール別箱化率
```
┌──────────────────────────────────────────────────────────────┐
│ モジュール別箱化状況                                          │
├──────────────────────────────────────────────────────────────┤
│ pipeline_v2/     100% ████████████████████████████████████   │
│ boxes/            83% ███████████████████████████░░░░░░░░░   │
│ parser/           50% ███████████████░░░░░░░░░░░░░░░░░░░░░   │
│ builder/          50% ███████████████░░░░░░░░░░░░░░░░░░░░░   │
│ emitter/          50% ███████████████░░░░░░░░░░░░░░░░░░░░░   │
│ mir/              50% ███████████████░░░░░░░░░░░░░░░░░░░░░   │
└──────────────────────────────────────────────────────────────┘
```

### Box品質スコア
```
┌──────────────────────────────────────────────────────────────┐
│ Box品質評価 (単一責任・サイズ・複雑度)                         │
├──────────────────────────────────────────────────────────────┤
│ Pipeline v2 Boxes 95% ████████████████████████████████████   │
│ Utility Boxes     90% ███████████████████████████████████░   │
│ Emitter Boxes     85% ██████████████████████████████████░░   │
│ Builder Boxes     70% ████████████████████████░░░░░░░░░░░░   │
│ ParserBox         30% ██████████░░░░░░░░░░░░░░░░░░░░░░░░░░   │
└──────────────────────────────────────────────────────────────┘
```

---

## ⚠️ 7. 問題のあるBox一覧

### 🔴 緊急対応が必要

#### 1. ParserBox (921行・59メソッド)

**問題点**:
- 単一責任の原則違反（レキシング・パース・using抽出・JSON生成を兼任）
- メソッド数が多すぎる（基準の6倍）
- テスト困難（モック作成が難しい）

**推奨対策**:
```
ParserBox を分割:
  ├─ LexerUtilsBox (is_digit, is_alpha, skip_ws)
  ├─ StringUtilsBox (trim, index_of, starts_with)
  ├─ ParserCoreBox (parse_program2 のみ)
  ├─ UsingExtractorBox (extract_usings, add_using)
  └─ JsonBuilderBox (JSON生成ヘルパー)
```

#### 2. LocalSSA (.nyash版, 547行)

**問題点**:
- .nyash形式（レガシー）
- 大きすぎる（100行超過の約5倍）
- .hako版（130行）は存在するが内容が異なる

**推奨対策**:
- .hako版に完全移行
- 必要に応じて機能を分割

#### 3. JsonProgramBox (520行)

**問題点**:
- サイズが大きい（JSON正規化ロジックが複雑）

**推奨対策**:
- JsonNormalizerBox + JsonMetaBox に分割検討

### 🟡 監視対象

#### 4. PipelineV2 (382行)
- flow定義のため許容範囲
- ただし、ロジックが増えたら分割検討

#### 5. Stage1ExtractFlow (206行)
- Extract系ロジックの複雑さが原因
- リファクタリング検討

---

## 💡 8. 改善推奨事項

### 優先度1: 緊急（1-2週間以内）

#### 1.1 ParserBox分割
```hakorune
// 現状 (ParserBox 921行)
box ParserBox {
  // 59メソッド...
}

// 改善後
static box LexerUtilsBox {
  is_digit(ch) -> bool
  is_alpha(ch) -> bool
  is_space(ch) -> bool
}

static box StringUtilsBox {
  trim(s) -> String
  index_of(src, start, pat) -> i64
  starts_with(src, pos, prefix) -> bool
}

box ParserBox {
  birth() { ... }
  parse_program2(src) -> String  // メインロジックのみ
}

static box UsingExtractorBox {
  extract_usings(src) -> Array
  add_using(kind, target, alias) -> i64
}
```

#### 1.2 共通ユーティリティBoxの作成

**apps/selfhost/common/utils/** に以下を新設:

1. `string_utils.hako` - 文字列操作
2. `number_utils.hako` - 数値変換
3. `json_utils.hako` - JSON操作

### 優先度2: 重要（1ヶ月以内）

#### 2.1 LocalSSA完全移行
- .nyash版（547行）→ .hako版に統合
- 必要に応じて機能分割

#### 2.2 JsonProgramBox分割
```hakorune
static box JsonNormalizerBox {
  normalize_program(json) -> String
  normalize_stmt_array(array_json) -> String
}

static box JsonMetaBox {
  ensure_meta(json, usings_json) -> String
}
```

#### 2.3 INTERFACES.md完全化
- Pipeline v2の全Boxを文書化
- Contractsセクションを充実

### 優先度3: 改善（2-3ヶ月以内）

#### 3.1 テストカバレッジ向上
- Stub Boxを活用した単体テスト追加
- 特にParserBox分割後のテスト整備

#### 3.2 エラーハンドリング統一
- すべてのBoxでFail-Fast原則適用
- 暗黙的フォールバックの排除

#### 3.3 ドキュメント整備
- 各Boxの責務をコメントで明記
- 使用例の追加

---

## 📊 9. 総合評価

### 総合スコア: **78/100** (良好)

| 評価項目 | スコア | コメント |
|---------|-------|---------|
| 箱化率 | 85/100 | .hako化が進んでいる |
| モジュール設計 | 80/100 | Pipeline v2は優秀、ParserBoxは問題 |
| コード品質 | 70/100 | 一部に肥大化が見られる |
| ベストプラクティス | 75/100 | 原則は守られているが例外あり |
| ドキュメント化 | 80/100 | INTERFACES.mdは優秀 |

### 強み ✅

1. **Pipeline v2の優れた設計**
   - 小さなBoxに適切に分割
   - using systemで依存関係を明示
   - 各BoxにStubが存在（テスト可能）

2. **箱化の徹底**
   - 75.4%が.hako形式
   - Everything is Box原則の実践

3. **インターフェース文書化**
   - INTERFACES.mdが整備
   - Fail-Fast原則の明示

4. **birth構文の適切な使用**
   - 必要な箇所のみ使用
   - 過剰な動的インスタンス化を避けている

### 弱み ⚠️

1. **ParserBoxの肥大化**
   - 921行・59メソッド（緊急対応必要）
   - 単一責任の原則違反

2. **コード重複**
   - trim, index_of, _to_i64などが複数箇所に重複
   - 共通ユーティリティBoxが未整備

3. **レガシーコード**
   - 14個の.nyashファイルが残存
   - LocalSSAの.nyash版（547行）が未移行

4. **ドキュメント不足**
   - Pipeline v2の多くのBoxが未文書化

---

## 🎯 10. アクションプラン

### Week 1-2: 緊急対応
- [ ] ParserBox分割設計
- [ ] 共通ユーティリティBox設計
- [ ] LocalSSA .hako移行計画策定

### Week 3-4: 実装
- [ ] StringUtilsBox実装
- [ ] NumberUtilsBox実装
- [ ] JsonUtilsBox実装
- [ ] ParserBox分割実装

### Month 2: 品質向上
- [ ] JsonProgramBox分割
- [ ] LocalSSA完全移行
- [ ] INTERFACES.md完全化
- [ ] テストカバレッジ向上

### Month 3: 仕上げ
- [ ] エラーハンドリング統一
- [ ] レガシー.nyash削除
- [ ] ドキュメント整備
- [ ] コードレビュー

---

## 📝 11. 結論

**総評**: Selfhost Compilerは全体として**良好な箱化・モジュール化**が実現されている。特にPipeline v2は**模範的な設計**であり、小さなBoxを組み合わせる「箱理論」の理想を体現している。

しかし、**ParserBoxの肥大化**と**共通ユーティリティの重複**が大きな問題となっており、これらの改善により**品質スコアを90点以上**に引き上げることが可能。

**推奨アクション**: ParserBox分割と共通ユーティリティBox作成を最優先で実施。これにより、コードベースの保守性・テスト性・再利用性が大幅に向上する。

---

**分析者**: Claude Code
**ツール**: Grep, Read, Bash
**方法論**: 箱理論4原則（箱にする・境界を作る・戻せる・見える化）
