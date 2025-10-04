# Selfhost Compiler 構造調査レポート

**調査日**: 2025-10-04
**調査対象**: `/home/tomoaki/git/hakorune-selfhost/apps/selfhost-compiler/`

---

## 📊 **概要統計**

- **総ファイル数**: 57ファイル (.hako/.nyash)
- **総行数**: 5,388行
- **ディレクトリ数**: 10個（サブディレクトリ含む）
- **主要言語**: Hakorune (.hako) / Nyash (.nyash)

---

## 🏗️ **全体アーキテクチャ**

### **パイプライン構造**

```
ソースコード
    ↓
[ParserBox] ────→ Stage-1 JSON (AST-ish)
    ↓
[Pipeline v2]
    ├─ Extract Boxes (抽出層)
    │   ├─ CompareExtractBox
    │   ├─ CallExtractBox
    │   ├─ MethodExtractBox
    │   └─ NewExtractBox
    │
    ├─ Normalizer Box (正規化層)
    │   └─ NormalizerBox
    │
    ├─ Emit Boxes (MIR生成層)
    │   ├─ EmitReturnBox
    │   ├─ EmitBinopBox
    │   ├─ EmitCompareBox
    │   ├─ EmitCallBox
    │   ├─ EmitMethodBox
    │   └─ EmitNewBoxBox
    │
    └─ SSA Box (SSA変換層)
        └─ LocalSSABox
    ↓
MIR JSON (v0/v1)
    ↓
[Backend] ────→ 実行 (Rust VM / PyVM / LLVM)
```

---

## 📁 **ディレクトリ構造詳細**

### **1. ルート (`/`)**
- **ファイル数**: 4ファイル
- **主要ファイル**:
  - `compiler.hako` (307行) - メインエントリポイント
  - `interfaces.hako/nyash` (23行×2) - インターフェース定義
- **役割**: コンパイラのメイン制御フロー

### **2. boxes/** (7ファイル, 2,038行)
| ファイル | 行数 | 役割 |
|---------|------|------|
| `parser_box.hako` | 921 | パーサー本体（Stage-1 JSON生成） |
| `json_program_box.hako` | 520 | JSON正規化・メタデータ追加 |
| `mir_emitter_box.hako/nyash` | 256×2 | MIR生成統括 |
| `debug_box.hako/nyash` | 38×2 | デバッグ支援 |
| `emitter_box.hako` | 9 | エミッター統括 |

**特徴**:
- `parser_box.hako`が最大（921行）→ **リファクタリング候補**
- JSONスキーマ正規化が`json_program_box.hako`に集約

### **3. pipeline_v2/** (24ファイル, 2,045行)
**Phase 15.7の中核**: Box-First Extract→Emit設計

#### **3.1 制御フロー (Flow)**
| ファイル | 行数 | 役割 |
|---------|------|------|
| `pipeline.hako` | 382 | メインパイプライン統合 |
| `stage1_extract_flow.hako` | 206 | レガシー抽出器 |
| `regex_flow.hako` | 103 | 正規表現風ヘルパー |
| `emit_mir_flow.hako` | 104 | MIR生成フロー |
| `emit_mir_flow_map.hako` | 151 | MIR生成（Map版） |

#### **3.2 抽出Boxes (Extract)**
| ファイル | 行数 | 役割 |
|---------|------|------|
| `compare_extract_box.hako` | 126 | Compare抽出（整数のみ） |
| `call_extract_box.hako` | 54 | Call抽出 |
| `method_extract_box.hako` | 51 | Method抽出 |
| `new_extract_box.hako` | 51 | New抽出 |

#### **3.3 生成Boxes (Emit)**
| ファイル | 行数 | 役割 |
|---------|------|------|
| `emit_compare_box.hako` | 66 | Compare MIR生成 |
| `emit_call_box.hako` | 75 | Call MIR生成 |
| `emit_method_box.hako` | 56 | Method MIR生成 |
| `emit_newbox_box.hako` | 49 | NewBox MIR生成 |
| `emit_binop_box.hako` | 33 | BinOp MIR生成 |
| `emit_return_box.hako` | 21 | Return MIR生成 |

#### **3.4 その他**
| ファイル | 行数 | 役割 |
|---------|------|------|
| `normalizer_box.hako` | 116 | 値正規化（型強制） |
| `local_ssa_box.hako` | 105 | LocalSSA変換 |
| `mir_call_box.hako` | 99 | MirCall生成 |
| `mir_builder_box.hako` | 34 | MIRビルダー統括 |
| `execution_pipeline_box.hako` | 37 | 実行パイプライン |
| `backend_box.hako` | 11 | バックエンド抽象化 |
| `map_helpers_box.hako` | 65 | Map操作ヘルパー |
| `readonly_map_view.hako` | 30 | 読み取り専用Mapビュー |
| `flow_entry.hako` | 20 | Flowエントリポイント |

### **4. builder/** (11ファイル, 886行)
**SSA変換・最適化レイヤー**

#### **4.1 SSAサブシステム**
| ファイル | 行数 | 役割 |
|---------|------|------|
| `ssa/local.nyash` | 547 | LocalSSA実装（最大） |
| `ssa/local.hako` | 130 | LocalSSA API |
| `ssa/cond_inserter.hako` | 117 | 条件挿入器 |
| `ssa/loopssa.hako/nyash` | 8×2 | ループSSA（スタブ） |

#### **4.2 リライト（最適化）**
| ファイル | 行数 | 役割 |
|---------|------|------|
| `rewrite/special.hako/nyash` | 8×2 | 特殊リライト |
| `rewrite/known.hako/nyash` | 8×2 | 既知パターンリライト |

#### **4.3 統括**
| ファイル | 行数 | 役割 |
|---------|------|------|
| `mod.hako/nyash` | 22×2 | ビルダーモジュール統括 |

**問題点**:
- `local.nyash`が547行で突出 → **分割候補**
- 他のファイルがほぼスタブ（8行）→ **実装不足**

### **5. emitter/** (2ファイル, 16行)
| ファイル | 行数 | 役割 |
|---------|------|------|
| `json_v0.hako/nyash` | 8×2 | JSON v0エミッター |

**問題点**: ほぼスタブ状態（実装は`boxes/`に集約）

### **6. mir/** (4ファイル, 20行)
| ファイル | 行数 | 役割 |
|---------|------|------|
| `builder.hako/nyash` | 5×2 | MIRビルダー |
| `optimizer.hako/nyash` | 5×2 | MIR最適化 |

**問題点**: 完全にスタブ（実装は`pipeline_v2/`に移行済み）

### **7. parser/** (6ファイル, 30行)
| ファイル | 行数 | 役割 |
|---------|------|------|
| `parser.hako/nyash` | 5×2 | パーサーAPI |
| `lexer.hako/nyash` | 5×2 | レキサーAPI |
| `ast.hako/nyash` | 5×2 | AST定義 |

**問題点**: 完全にスタブ（実装は`boxes/parser_box.hako`に集約）

### **8. tests/** (2ディレクトリ)
- `tests/stage1/` - Stage-1テスト用
- READMEのみ存在（実装ファイルなし）

---

## 🔄 **依存関係マップ**

### **主要Box定義一覧** (30個)

#### **1. コアBox (6個)**
```
ParserBox          - パーサー本体（921行）
EmitterBox         - エミッター統括（9行）
MirEmitterBox      - MIR生成統括（256行×2）
JsonProgramBox     - JSON正規化（520行）
DebugBox          - デバッグ支援（38行×2）
Main              - エントリポイント（compiler.hako内）
```

#### **2. Pipeline v2 Box (16個)**
```
ExecutionPipelineBox - 実行パイプライン
BackendBox           - バックエンド抽象化
MirBuilderBox        - MIRビルダー統括
LocalSSABox          - LocalSSA変換

[Extract Boxes - 4個]
CompareExtractBox    - Compare抽出
CallExtractBox       - Call抽出
MethodExtractBox     - Method抽出
NewExtractBox        - New抽出

[Emit Boxes - 6個]
EmitReturnBox        - Return生成
EmitBinopBox         - BinOp生成
EmitCompareBox       - Compare生成
EmitCallBox          - Call生成
EmitMethodBox        - Method生成
EmitNewBoxBox        - NewBox生成

[Utility Boxes - 2個]
NormalizerBox        - 値正規化
MapHelpersBox        - Map操作
ReadOnlyMapView      - 読み取り専用Map
MirCallBox           - MirCall生成
```

#### **3. Builder Box (6個)**
```
CompilerBuilder      - ビルダー統括
LocalSSA            - LocalSSA実装（547行）
CondInserter        - 条件挿入
LoopSSA             - ループSSA（スタブ）
RewriteSpecial      - 特殊リライト
RewriteKnown        - 既知リライト
```

#### **4. Legacy Stub Box (8個)**
```
MirBuilder          - MIRビルダー（レガシー）
Optimizer           - 最適化（レガシー）
Parser              - パーサー（レガシー）
Lexer               - レキサー（レガシー）
AST                 - AST（レガシー）
JsonV0Emitter       - JSON v0エミッター
```

### **主要Flow定義** (5個)
```
PipelineV2          - メインパイプライン（382行）
Stage1ExtractFlow   - レガシー抽出器（206行）
RegexFlow           - 正規表現ヘルパー（103行）
EmitMirFlow         - MIR生成フロー（104行）
EmitMirFlowMap      - MIR生成Map版（151行）
```

### **依存関係グラフ** (主要モジュールのみ)

```
compiler.hako (Main)
    │
    ├─→ ParserBoxMod (boxes/parser_box.hako)
    ├─→ JsonProgramBox (boxes/json_program_box.hako)
    ├─→ EmitterBoxMod (boxes/emitter_box.hako)
    ├─→ ExecutionPipelineBox (pipeline_v2/)
    └─→ PipelineV2 (pipeline_v2/pipeline.hako)
            │
            ├─→ Stage1ExtractFlow
            ├─→ CompareExtractBox ─→ RegexFlow
            ├─→ CallExtractBox ─→ RegexFlow
            ├─→ MethodExtractBox ─→ RegexFlow
            ├─→ NewExtractBox ─→ RegexFlow
            ├─→ NormalizerBox
            ├─→ EmitReturnBox
            ├─→ EmitBinopBox
            ├─→ EmitCompareBox ─→ MirJsonBuilder2
            ├─→ EmitCallBox ─→ MirJsonBuilderMin
            ├─→ EmitMethodBox ─→ MirJsonBuilderMin
            ├─→ EmitNewBoxBox ─→ MirJsonBuilderMin
            ├─→ MirCallBox ─→ MirJsonBuilderMin
            ├─→ LocalSSA (builder/ssa/local.hako)
            └─→ MirJsonV1Adapter (外部: apps/selfhost/common/json/)

ExecutionPipelineBox
    │
    ├─→ ParserBoxMod
    ├─→ JsonProgramBox
    ├─→ EmitterBoxMod
    └─→ BackendBox

builder/mod.hako (CompilerBuilder)
    │
    ├─→ LocalSSA (builder/ssa/local.hako)
    ├─→ LoopSSA (builder/ssa/loopssa.hako)
    ├─→ RewriteSpecial (builder/rewrite/special.hako)
    └─→ RewriteKnown (builder/rewrite/known.hako)

boxes/mir_emitter_box.hako (MirEmitterBox)
    │
    ├─→ MirJsonBuilderMin (外部: apps/selfhost/common/json/)
    └─→ JSON (外部: apps/lib/json_native/stringify.hako)
```

### **外部依存** (3箇所)
```
selfhost.common.json.mir_builder_min     - MIR JSON生成（最小版）
selfhost.common.json.mir_v1_adapter      - MIR v1/v0変換
apps/lib/json_native/stringify.hako      - JSON文字列化
apps/selfhost/common/json/mir_builder2   - MIR JSON生成（v2）
apps/selfhost/vm/boxes/json_frag.hako    - JSON断片操作
```

---

## ⚠️ **問題点・改善候補**

### **1. コード集中問題** (行数肥大化)

#### **超大型ファイル**
- `boxes/parser_box.hako` (921行)
  - **問題**: パーサーロジック・レキサー・JSON生成が混在
  - **提案**: レキサー分離、JSON生成を`json_program_box.hako`へ移行

- `builder/ssa/local.nyash` (547行)
  - **問題**: LocalSSA実装が単一ファイルに集約
  - **提案**: 機能別に分割（Copy挿入、PHI処理、値追跡等）

#### **中型ファイル** (300行超)
- `boxes/json_program_box.hako` (520行) - 機能別分割検討
- `pipeline_v2/pipeline.hako` (382行) - Extract/Emit統合ロジックが混在

### **2. スタブ問題** (未実装・重複)

#### **完全スタブディレクトリ**
- `mir/` (20行) - 全ファイルが5行スタブ → **削除候補**
- `parser/` (30行) - 全ファイルが5行スタブ → **削除候補**
- `emitter/` (16行) - 全ファイルが8行スタブ → **統合候補**

#### **重複定義**
- `ParserBox`: `boxes/parser_box.hako` (実装) vs `interfaces.hako` (仕様) vs `parser/parser.hako` (スタブ)
- `MirBuilder`: `pipeline_v2/mir_builder_box.hako` (実装) vs `mir/builder.hako` (スタブ)
- `Optimizer`: `mir/optimizer.hako` (スタブのみ、実装なし)

### **3. 責務分離問題**

#### **Extract層の不統一**
- 新Extract Boxes: `CompareExtractBox`, `CallExtractBox` 等 (robustな実装)
- レガシー: `Stage1ExtractFlow` (fallback用、206行)
- **提案**: レガシー抽出器を段階的に廃止、Extract Boxに完全移行

#### **Emit層の散在**
- Emit Boxes: `emit_*_box.hako` (6個)
- 統括: `EmitMirFlow`, `EmitMirFlowMap` (2個)
- **提案**: Emit統括を単一Flowに統合

#### **SSA層の分散**
- `builder/ssa/local.hako` (API)
- `builder/ssa/local.nyash` (実装、547行)
- `pipeline_v2/local_ssa_box.hako` (Box版、105行)
- **問題**: 同じ機能が2箇所に分散 → **統合候補**

### **4. 循環依存リスク**

現時点で循環依存は**検出されず**（健全な設計）。ただし注意点:
- `PipelineV2` → `LocalSSA` → `PipelineV2` (将来的リスク)
- 現在は`LocalSSA`がNo-Op APIのため問題なし

### **5. テスト不足**
- `tests/` ディレクトリが存在するが、実装ファイルなし
- **提案**: 各Extract/Emit Boxのユニットテスト追加

---

## 🎯 **設計パターン分析**

### **良い点** ✅

1. **Box-First設計**
   - 各機能がBoxとして独立（Extract/Emit/Normalize）
   - 責務が明確（Extract→Normalize→Emit）

2. **Flow/Box分離**
   - Flow: 制御フロー（PipelineV2, Stage1ExtractFlow）
   - Box: 機能単位（Extract/Emit各Box）

3. **段階的移行戦略**
   - レガシー（Stage1ExtractFlow）→ 新Extract Boxes
   - fallback機構で後方互換性維持

4. **外部依存の最小化**
   - 外部依存は4箇所のみ（JSON処理系のみ）

5. **Fail-Fast設計**
   - Extract Boxは失敗時null返却
   - pipelineでfallback処理

### **改善点** ⚠️

1. **ファイル粒度の不均一**
   - 最大921行 vs 最小5行（スタブ）
   - 200-300行を目安に分割推奨

2. **スタブの過剰残存**
   - `mir/`, `parser/`, `emitter/` が完全スタブ
   - 削除 or 統合検討

3. **SSA実装の重複**
   - `builder/ssa/` vs `pipeline_v2/local_ssa_box.hako`
   - 統一API化推奨

4. **ドキュメントとコードの乖離**
   - `interfaces.hako` の仕様 vs 実装の不一致
   - インターフェース定義の更新が必要

---

## 📋 **推奨アクション**

### **優先度1: 即座に実施**
1. **スタブディレクトリ削除**
   - `mir/` (20行) → 削除
   - `parser/` (30行) → 削除
   - `emitter/` (16行) → `boxes/emitter_box.hako`へ統合

2. **超大型ファイル分割**
   - `parser_box.hako` (921行) → レキサー分離 + JSON生成移行
   - `local.nyash` (547行) → 機能別3-4ファイルへ分割

### **優先度2: 段階的実施**
3. **Extract層統一**
   - `Stage1ExtractFlow` (206行) の段階的廃止
   - 全機能をExtract Boxへ移行

4. **SSA層統合**
   - `builder/ssa/` と `pipeline_v2/local_ssa_box.hako` の統合
   - 統一APIの確立

5. **Emit層整理**
   - `EmitMirFlow` と `EmitMirFlowMap` の統合
   - 単一Emit統括Flowへ集約

### **優先度3: 長期改善**
6. **テストインフラ構築**
   - 各Extract/Emit Boxのユニットテスト追加
   - `tests/` ディレクトリ活用

7. **インターフェース定義更新**
   - `interfaces.hako` を実装と同期
   - 型注釈・契約の明確化

8. **ドキュメント整備**
   - 各Boxの責務・I/O仕様を明文化
   - READMEの更新（現在の設計に合わせる）

---

## 📐 **モジュール関係図** (ASCII)

```
                    ┌──────────────┐
                    │ compiler.hako│
                    │   (Main)     │
                    └───────┬──────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ▼                   ▼                   ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│ ParserBox    │    │ExecutionPipe │    │ PipelineV2   │
│ (921 lines)  │    │   lineBox    │    │ (382 lines)  │
└──────────────┘    └──────────────┘    └───────┬──────┘
                                                 │
                ┌────────────────────────────────┼────────────────┐
                │                                │                │
                ▼                                ▼                ▼
        ┌──────────────┐              ┌──────────────┐  ┌──────────────┐
        │ Extract Boxes│              │ Emit Boxes   │  │ Utility Boxes│
        │  (4 boxes)   │              │  (6 boxes)   │  │  (4 boxes)   │
        └──────────────┘              └──────────────┘  └──────────────┘
        │                              │                │
        ├─ CompareExtract              ├─ EmitReturn   ├─ Normalizer
        ├─ CallExtract                 ├─ EmitBinop    ├─ LocalSSA
        ├─ MethodExtract               ├─ EmitCompare  ├─ MapHelpers
        └─ NewExtract                  ├─ EmitCall     └─ MirCall
                                       ├─ EmitMethod
                                       └─ EmitNewBox

        ┌──────────────┐
        │ RegexFlow    │ ←──── (全Extract Boxから参照)
        │ (103 lines)  │
        └──────────────┘

        ┌──────────────┐
        │ builder/ssa/ │
        │ LocalSSA     │ ←──── (PipelineV2から参照)
        │ (547 lines)  │
        └──────────────┘

        ┌──────────────┐
        │ External     │
        │ - MirBuilder │ ←──── (Emit Boxesから参照)
        │ - JSON       │
        └──────────────┘
```

---

## 🔍 **Box一覧表** (機能別)

### **1. Parser Layer (1 box)**
| Box | File | Lines | Status |
|-----|------|-------|--------|
| ParserBox | boxes/parser_box.hako | 921 | ✅ 実装済（分割推奨） |

### **2. Pipeline Layer (4 boxes)**
| Box | File | Lines | Status |
|-----|------|-------|--------|
| ExecutionPipelineBox | pipeline_v2/execution_pipeline_box.hako | 37 | ✅ 実装済 |
| BackendBox | pipeline_v2/backend_box.hako | 11 | ⚠️ スタブ |
| MirBuilderBox | pipeline_v2/mir_builder_box.hako | 34 | ✅ 実装済 |
| FlowEntryBox | pipeline_v2/flow_entry.hako | 20 | ✅ 実装済 |

### **3. Extract Layer (4 boxes)**
| Box | File | Lines | Status |
|-----|------|-------|--------|
| CompareExtractBox | pipeline_v2/compare_extract_box.hako | 126 | ✅ 実装済 |
| CallExtractBox | pipeline_v2/call_extract_box.hako | 54 | ✅ 実装済 |
| MethodExtractBox | pipeline_v2/method_extract_box.hako | 51 | ✅ 実装済 |
| NewExtractBox | pipeline_v2/new_extract_box.hako | 51 | ✅ 実装済 |

### **4. Emit Layer (6 boxes)**
| Box | File | Lines | Status |
|-----|------|-------|--------|
| EmitCompareBox | pipeline_v2/emit_compare_box.hako | 66 | ✅ 実装済 |
| EmitCallBox | pipeline_v2/emit_call_box.hako | 75 | ✅ 実装済 |
| EmitMethodBox | pipeline_v2/emit_method_box.hako | 56 | ✅ 実装済 |
| EmitNewBoxBox | pipeline_v2/emit_newbox_box.hako | 49 | ✅ 実装済 |
| EmitBinopBox | pipeline_v2/emit_binop_box.hako | 33 | ✅ 実装済 |
| EmitReturnBox | pipeline_v2/emit_return_box.hako | 21 | ✅ 実装済 |

### **5. Utility Layer (6 boxes)**
| Box | File | Lines | Status |
|-----|------|-------|--------|
| NormalizerBox | pipeline_v2/normalizer_box.hako | 116 | ✅ 実装済 |
| LocalSSABox | pipeline_v2/local_ssa_box.hako | 105 | ✅ 実装済 |
| MirCallBox | pipeline_v2/mir_call_box.hako | 99 | ✅ 実装済 |
| MapHelpersBox | pipeline_v2/map_helpers_box.hako | 65 | ✅ 実装済 |
| ReadOnlyMapView | pipeline_v2/readonly_map_view.hako | 30 | ✅ 実装済 |
| JsonProgramBox | boxes/json_program_box.hako | 520 | ✅ 実装済（分割検討） |

### **6. Builder Layer (6 boxes)**
| Box | File | Lines | Status |
|-----|------|-------|--------|
| LocalSSA | builder/ssa/local.hako | 130 | ✅ 実装済 |
| LocalSSA (impl) | builder/ssa/local.nyash | 547 | ✅ 実装済（分割推奨） |
| CondInserter | builder/ssa/cond_inserter.hako | 117 | ✅ 実装済 |
| LoopSSA | builder/ssa/loopssa.hako | 8 | ⚠️ スタブ |
| RewriteSpecial | builder/rewrite/special.hako | 8 | ⚠️ スタブ |
| RewriteKnown | builder/rewrite/known.hako | 8 | ⚠️ スタブ |
| CompilerBuilder | builder/mod.hako | 22 | ✅ 実装済 |

### **7. Emitter Layer (3 boxes)**
| Box | File | Lines | Status |
|-----|------|-------|--------|
| EmitterBox | boxes/emitter_box.hako | 9 | ✅ 実装済 |
| MirEmitterBox | boxes/mir_emitter_box.hako | 256 | ✅ 実装済 |
| JsonV0Emitter | emitter/json_v0.hako | 8 | ⚠️ スタブ |

### **8. Debug Layer (1 box)**
| Box | File | Lines | Status |
|-----|------|-------|--------|
| DebugBox | boxes/debug_box.hako | 38 | ✅ 実装済 |

### **9. Legacy/Stub Boxes (6 boxes)** ❌ 削除候補
| Box | File | Lines | Status |
|-----|------|-------|--------|
| MirBuilder | mir/builder.hako | 5 | ❌ スタブ（削除候補） |
| Optimizer | mir/optimizer.hako | 5 | ❌ スタブ（削除候補） |
| Parser | parser/parser.hako | 5 | ❌ スタブ（削除候補） |
| Lexer | parser/lexer.hako | 5 | ❌ スタブ（削除候補） |
| AST | parser/ast.hako | 5 | ❌ スタブ（削除候補） |

---

## 📊 **Flow一覧表**

| Flow | File | Lines | 役割 | Status |
|------|------|-------|------|--------|
| PipelineV2 | pipeline_v2/pipeline.hako | 382 | メインパイプライン統合 | ✅ 実装済 |
| Stage1ExtractFlow | pipeline_v2/stage1_extract_flow.hako | 206 | レガシー抽出器（fallback） | ⚠️ 段階廃止中 |
| RegexFlow | pipeline_v2/regex_flow.hako | 103 | 正規表現風ヘルパー | ✅ 実装済 |
| EmitMirFlow | pipeline_v2/emit_mir_flow.hako | 104 | MIR生成フロー | ✅ 実装済 |
| EmitMirFlowMap | pipeline_v2/emit_mir_flow_map.hako | 151 | MIR生成（Map版） | ✅ 実装済 |

---

## 🎯 **コード品質メトリクス**

### **ファイルサイズ分布**
```
0-50行:    28ファイル (49%) - スタブ・小型モジュール
51-100行:   14ファイル (25%) - 適正サイズ
101-200行:   8ファイル (14%) - やや大きめ
201-500行:   4ファイル (7%)  - 大型（分割検討）
501行以上:   3ファイル (5%)  - 超大型（要分割）
```

### **実装状態**
```
✅ 完全実装: 38ファイル (67%)
⚠️ スタブ:   14ファイル (25%)
❌ 削除候補:  5ファイル (9%)
```

### **責務の明確度**
```
高: Extract Boxes, Emit Boxes (明確な単一責務)
中: Pipeline統合層 (複数機能の配線)
低: builder/ssa/local.nyash (547行、多機能混在)
```

---

## 🔧 **リファクタリング優先度マトリックス**

| 項目 | 影響度 | 工数 | 優先度 | アクション |
|------|--------|------|--------|-----------|
| スタブディレクトリ削除 (mir/, parser/, emitter/) | 低 | 小 | 🔴 高 | 即実施 |
| parser_box.hako 分割 (921行) | 高 | 中 | 🔴 高 | Phase 1 |
| local.nyash 分割 (547行) | 中 | 中 | 🟡 中 | Phase 2 |
| Stage1ExtractFlow 廃止 | 中 | 大 | 🟡 中 | Phase 3 |
| SSA層統合 (builder vs pipeline_v2) | 中 | 大 | 🟡 中 | Phase 4 |
| Emit層統合 (Flow整理) | 低 | 小 | 🟢 低 | Phase 5 |
| テスト追加 | 高 | 大 | 🟡 中 | 並行実施 |
| インターフェース定義更新 | 低 | 小 | 🟢 低 | ドキュメント作業 |

---

## 📝 **まとめ**

### **全体評価**: 🟡 **良好だが改善余地あり**

#### **強み** ✅
1. **Box-First設計が確立** - 責務分離が明確
2. **段階的移行戦略** - レガシー→新Extract Boxsへの移行計画
3. **外部依存の最小化** - JSON処理系のみ依存
4. **循環依存なし** - 健全な依存グラフ
5. **Fail-Fast設計** - エラーハンドリング明確

#### **弱み** ⚠️
1. **ファイル粒度の不均一** - 921行 vs 5行（スタブ）
2. **スタブの過剰残存** - 3ディレクトリが完全スタブ
3. **SSA実装の重複** - builder/ vs pipeline_v2/
4. **超大型ファイル** - 2ファイルが500行超
5. **テスト不足** - tests/に実装なし

#### **推奨方針**
1. **即座に**: スタブディレクトリ削除（影響小、効果大）
2. **短期**: 超大型ファイル分割（parser_box, local.nyash）
3. **中期**: Extract層統一、SSA層統合
4. **長期**: テストインフラ構築、ドキュメント整備

---

**調査者コメント**:
全体的に**Box-First設計が成功**しており、責務分離が明確。ただし、**初期開発の痕跡**（スタブ、超大型ファイル）が残存。**段階的リファクタリング**により、さらに保守性の高いアーキテクチャに進化可能。特に`parser_box.hako` (921行)と`local.nyash` (547行)の分割が最優先事項。
