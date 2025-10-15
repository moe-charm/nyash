# MIR Builder系 アーキテクチャ図

## 現状アーキテクチャ（問題あり）

```
┌─────────────────────────────────────────────────────────┐
│ Pipeline Layer (orchestration)                          │
├─────────────────────────────────────────────────────────┤
│ • CompilerBuilder.apply_all()                           │
│ • EmitCompareBox, EmitBinopBox, EmitReturnBox           │
└──────────────┬──────────────────────────────────────────┘
               │
┌──────────────▼──────────────────────────────────────────┐
│ Builder Layer (MIR construction) ⚠️ 重複あり             │
├─────────────────────────────────────────────────────────┤
│ • MirJsonBuilderMin (437行)                             │
│   - state: fields直接                                   │
│   - trace/verify あり                                   │
│ • MirJsonBuilder2 (160行)                               │
│   - state: MapBox経由                                   │
│   - prefer_rebuild あり                                 │
│                                                          │
│ 🔴 問題: 状態管理が2つのBuilderで重複（40-60行）         │
└──────────────┬──────────────────────────────────────────┘
               │
┌──────────────▼──────────────────────────────────────────┐
│ SSA Transform Layer ⚠️ 実装なし（no-op）                 │
├─────────────────────────────────────────────────────────┤
│ • LocalSSA (122行) - 5つのno-op公開API                  │
│   - 内部ヘルパーは多数あるが未使用                        │
│ • LoopSSA (9行) - 完全no-op                             │
│ • CondInserter (118行) - 唯一の実装あり                 │
│                                                          │
│ 🔴 問題: Box化されていない、拡張性なし                   │
└──────────────┬──────────────────────────────────────────┘
               │
┌──────────────▼──────────────────────────────────────────┐
│ Utility Layer 🔴 重複・散在                              │
├─────────────────────────────────────────────────────────┤
│ • StringHelpers (87行)                                  │
│ • JsonFragBox (75行)                                    │
│ • JsonCursorBox (39行)                                  │
│                                                          │
│ 🔴 重複実装箇所:                                         │
│   - local.hako: _index_of, _seek_obj_end (27行)        │
│   - cond_inserter.hako: _seek_obj_end (30行、独自版)    │
│   - stage1_extract_flow.hako: _idx, _idx_from (206行)  │
│                                                          │
│ 🔴 問題: JSON文字列パースが3ファイルに重複（80-100行）    │
└─────────────────────────────────────────────────────────┘
```

---

## 推奨アーキテクチャ（箱化後）

```
┌─────────────────────────────────────────────────────────┐
│ Pipeline Layer (orchestration)                          │
├─────────────────────────────────────────────────────────┤
│ • CompilerBuilder.apply_all()                           │
│ • EmitCompareBox, EmitBinopBox, EmitReturnBox           │
│ • CompilerConfigBox (新規) ← 設定統一管理               │
└──────────────┬──────────────────────────────────────────┘
               │
┌──────────────▼──────────────────────────────────────────┐
│ Builder Layer (MIR construction) ✅ 統一                 │
├─────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────┐ │
│ │ MirBuilderContext (新規) ← 状態管理統一             │ │
│ │   buf, phase, blocks, cur_block_index, fn_name      │ │
│ │   trace_enabled, verify_enabled, rebuild_mode       │ │
│ └─────────────────────────────────────────────────────┘ │
│                                                          │
│ • MirJsonBuilderMin (Context使用版)                     │
│ • MirJsonBuilder2 (Context使用版)                       │
│                                                          │
│ ✅ 改善: 状態管理を1箇所に統一（40-60行削減）            │
└──────────────┬──────────────────────────────────────────┘
               │
┌──────────────▼──────────────────────────────────────────┐
│ SSA Transform Layer ✅ Box化、拡張可能                   │
├─────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────┐ │
│ │ SSATransformBase (新規基底Box)                      │ │
│ │   name, enabled                                     │ │
│ │   transform(mir_json) - 統一インターフェース         │ │
│ └─────────────────────────────────────────────────────┘ │
│                  ▲         ▲         ▲                  │
│                  │         │         │                  │
│   ┌──────────────┘         │         └──────────────┐   │
│   │                        │                        │   │
│ LocalSSA              CondCopyInsert            LoopSSA │
│ Transform             Transform                Transform│
│                                                          │
│ ✅ 改善: 各変換がBoxとして独立、拡張容易                 │
└──────────────┬──────────────────────────────────────────┘
               │
┌──────────────▼──────────────────────────────────────────┐
│ Utility Layer ✅ 統一・箱化                              │
├─────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────┐ │
│ │ JsonStringParserBox (新規) ← JSON文字列パース統一    │ │
│ │   find_key, extract_int/str, seek_object/array_end  │ │
│ └─────────────────────────────────────────────────────┘ │
│                                                          │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ Stage1AstExtractorBox (新規) ← AST抽出統一          │ │
│ │   extract_return_int/binop/compare/method/new/call  │ │
│ └─────────────────────────────────────────────────────┘ │
│                                                          │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ StringBuilderBox (新規、Optional) ← 性能改善        │ │
│ │   append(), to_string()                             │ │
│ └─────────────────────────────────────────────────────┘ │
│                                                          │
│ • StringHelpers (既存)                                  │
│ • JsonFragBox (既存)                                    │
│ • JsonCursorBox (既存)                                  │
│                                                          │
│ ✅ 改善: 重複削減（80-100行）、保守性向上               │
└─────────────────────────────────────────────────────────┘
```

---

## データフロー図

### 現状（複雑・重複あり）

```
Hakorune Source (.hako)
         │
         ▼
    [Parser] (Rust実装)
         │
         ▼
    Stage1 AST (JSON)
         │
         ▼──────────────────────────────────┐
         │                                   │
         ▼                                   ▼
  [stage1_extract_flow]              [MirBuilderBox]
  🔴 独自JSON文字列パース                  │
  - _idx, _idx_from (重複)                 │
  - extract_xxx() 7関数                    │
         │                                   │
         ▼                                   ▼
    AST情報抽出                  ┌─────────────────┐
         │                      │ MirJsonBuilder  │
         └──────────────────────►│ Min / Builder2  │
                                 │ 🔴 状態管理重複  │
                                 └────────┬────────┘
                                          │
                                          ▼
                                     MIR (JSON)
                                          │
                                          ▼
         ┌────────────────────────────────┤
         │                                │
         ▼                                ▼
  [LocalSSA]                      [CondInserter]
  🔴 no-op（実装なし）              🔴 独自JSON文字列パース
  - 内部ヘルパー未使用                  - _seek_obj_end (重複)
         │                                │
         ▼                                ▼
    MIR (unchanged)              MIR (cond copy挿入)
         │                                │
         └────────────┬───────────────────┘
                      │
                      ▼
                  最終MIR (JSON)
                      │
                      ▼
                 [Rust VM / LLVM]
```

**問題点**:
- 🔴 JSON文字列パース処理が3箇所に重複
- 🔴 Builder状態管理が2箇所に重複
- 🔴 SSA変換がBoxとして独立していない

---

### 推奨（シンプル・統一）

```
Hakorune Source (.hako)
         │
         ▼
    [Parser] (Rust実装)
         │
         ▼
    Stage1 AST (JSON)
         │
         ▼
  [Stage1AstExtractorBox] ✅ 新規統一Box
  - JsonStringParserBox使用 ✅ 共通化
  - extract_xxx() 統一API
         │
         ▼
    AST情報抽出
         │
         ▼
  [MirJsonBuilderMin/2] ✅ Context使用
  - MirBuilderContext ✅ 状態管理統一
  - StringBuilderBox ✅ 性能改善（Optional）
         │
         ▼
    MIR (JSON)
         │
         ▼
  [SSATransformPipeline] ✅ Box化
  - LocalSSATransform ✅ 独立Box
  - CondCopyInsertTransform ✅ 独立Box
  - LoopSSATransform ✅ 独立Box（実装待ち）
         │
         ▼
    最終MIR (JSON)
         │
         ▼
  [Rust VM / LLVM]
```

**改善点**:
- ✅ JSON文字列パース: 1箇所に統一（JsonStringParserBox）
- ✅ Builder状態管理: 1箇所に統一（MirBuilderContext）
- ✅ SSA変換: 各Boxが独立、拡張容易

---

## 重複コード削減マップ

### Before（重複あり）

```
apps/selfhost-compiler/
├── builder/ssa/
│   ├── local.hako (122行)
│   │   ├── _index_of, _index_of_from     🔴 重複
│   │   ├── _seek_obj_end, _seek_array_end 🔴 重複
│   │   └── ensure_xxx() - 5つのno-op
│   ├── loopssa.hako (9行) - 完全no-op
│   └── cond_inserter.hako (118行)
│       ├── _index_of_from, _read_digits  🔴 重複（委譲版）
│       └── _seek_obj_end (独自実装)      🔴 重複（エスケープ版）
│
├── pipeline_v2/
│   ├── stage1_extract_flow.hako (206行)
│   │   ├── _idx, _idx_from               🔴 重複
│   │   └── extract_xxx() 7関数           🔴 類似パターン
│   └── mir_builder_box.hako (35行)
│
└── common/json/
    ├── mir_builder2.hako (160行)
    │   ├── state: MapBox経由             🔴 重複
    │   └── add_xxx() メソッド            🔴 重複
    └── mir_builder_min.hako (437行)
        ├── state: フィールド直接          🔴 重複
        └── add_xxx() メソッド            🔴 重複

重複合計: 200-280行（推定）
```

### After（統一後）

```
apps/selfhost-compiler/
├── builder/ssa/
│   ├── ssa_transform_base.hako ✅ 新規
│   ├── local_ssa_transform.hako ✅ Box化
│   ├── loop_ssa_transform.hako ✅ Box化
│   └── cond_copy_insert_transform.hako ✅ Box化
│
├── pipeline_v2/
│   └── stage1_ast_extractor_box.hako ✅ 新規統一Box
│
└── common/
    ├── json/
    │   ├── json_string_parser_box.hako ✅ 新規統一Box
    │   ├── mir_builder_context.hako ✅ 新規統一Box
    │   ├── mir_builder2.hako (Context使用版)
    │   └── mir_builder_min.hako (Context使用版)
    └── string_builder_box.hako ✅ 新規（Optional）

削減: 200-280行
追加: 新Box実装（200-300行）
正味: 保守性向上、重複削減、拡張性向上
```

---

## 優先度マトリクス

```
               効果（行削減）
                   │
        高         │
                   │
    JsonString     │  MirBuilderContext
    ParserBox      │  (40-60行)
    (80-100行)     │
        ▲          │
        │          │
        │          ├──────────────────
        │          │
        │          │  Stage1AstExtractor
        │          │  (60-80行)
───────┼──────────┼─────────────────► 工数
        │          │
        │          │  SSATransformBase
        │          │  (20-40行)
        │          │
        低         │  StringBuilderBox
                   │  (性能改善)
                   │
```

**推奨順序**:
1. JsonStringParserBox（最大効果）
2. MirBuilderContext（重要度高）
3. Stage1AstExtractorBox（中期）
4. SSATransformBase（中期）
5. StringBuilderBox（性能要求時）

---

## まとめ

### 現状の問題（3つの重複）

1. **JSON文字列パース** - 3ファイルに重複（80-100行）
2. **Builder状態管理** - 2ファイルに重複（40-60行）
3. **Stage1抽出パターン** - 7関数で類似（60-80行）

### 推奨実装（2週間で完了）

1. **JsonStringParserBox** ✅ 最優先
2. **MirBuilderContext** ✅ 2番目

→ **120-160行削減**、保守性・拡張性大幅向上

---

**作成日**: 2025-10-12
**関連ドキュメント**: [mir-builder-boxification-analysis.md](./mir-builder-boxification-analysis.md)
