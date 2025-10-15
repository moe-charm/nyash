# モジュール化計画レポート

**分析日**: 2025-10-15
**対象**: selfhost/ ディレクトリ構造
**総ファイル数**: 165 .hako ファイル (26,834行)

---

## エグゼクティブサマリー

selfhost/ ディレクトリには **2つの完全に独立したVM実装** が存在し、構造的な重複と混乱を招いています：

1. **`vm/` (旧Mini-VM)**: 30ファイル、1,600行程度
2. **`hakorune-vm/` (新Hakorune VM)**: 67ファイル、3,446行、22命令ハンドラー実装済み

**重大な発見**: これらは別々のプロジェクトとして進化しており、統一が必要です。

---

## 現状の問題点

### 🔴 **深刻度: 高** - VM実装の重複と混乱

#### 問題1: 2つのVM実装が並存
```
selfhost/
├── vm/                    # 旧Mini-VM (5ファイル + boxes/25ファイル)
│   ├── boxes/mini_vm_core.hako
│   ├── boxes/mir_vm_min.hako
│   └── boxes/mini_vm_entry.hako
└── hakorune-vm/          # 新Hakorune VM (44ファイル + tests/22)
    ├── hakorune_vm_core.hako
    ├── instruction_dispatcher.hako
    ├── *_handler.hako (22種類)
    └── tests/ (22ファイル)
```

**影響**:
- 開発者が「どちらを使うべきか」判断できない
- `using selfhost.vm.*` と `using "selfhost/hakorune-vm/*"` が混在
- テストが分散 (vm/には統合テストなし、hakorune-vm/testsには22個)

#### 問題2: 命名の不統一

| パターン | 例 | 出現数 |
|---------|-----|--------|
| `*_handler.hako` | `binop_handler.hako` | 17個 (hakorune-vm) |
| `*_box.hako` | `mir_builder_box.hako` | 38個 (compiler, shared) |
| `*_guard.hako` | `args_guard.hako`, `reg_guard.hako` | 3個 (hakorune-vm) |
| `*_locator.hako` | `function_locator.hako` | 3個 (hakorune-vm) |
| `*_extractor.hako` | `args_extractor.hako` | 2個 (hakorune-vm) |

**問題**: `*_box` サフィックスは compiler/pipeline_v2 で使用され、hakorune-vm では使われない。統一性なし。

#### 問題3: モジュール境界の不明確さ

**shared/ の役割混乱**:
```
shared/
├── common/          # VM共通ヘルパー (6ファイル)
│   ├── mini_vm_scan.hako        # 旧VM用
│   ├── mini_vm_binop.hako       # 旧VM用
│   ├── mini_vm_compare.hako     # 旧VM用
│   ├── string_helpers.hako      # 汎用？
│   └── string_ops.hako          # 汎用？
├── json/            # JSON処理 (7ファイル)
├── mir/             # MIR構築 (3ファイル)
├── adapters/        # アダプター (1ファイル)
├── backend/         # LLVM backend (1ファイル)
└── host_bridge/     # ホストブリッジ (1ファイル)
```

**問題**:
- `mini_vm_*` は旧VMに依存しているが `shared/` にある
- `string_helpers` は汎用だが `common/` に埋もれている
- JSON処理が3箇所に分散: `shared/json/`, `hakorune-vm/json_*.hako`, `vm/boxes/json_cur.hako`

### 🟡 **深刻度: 中** - ディレクトリ階層の深さ

```
selfhost/shared/json/core/json_scan.hako           # 4階層
selfhost/hakorune-vm/boxes/handlers/               # 4階層 (空)
selfhost/compiler/pipeline_v2/stage1_*.hako        # 3階層
```

**問題**:
- `hakorune-vm/boxes/handlers/` は空ディレクトリ
- `shared/json/core/` と `shared/json/utils/` は2-3ファイルしかない (過剰な細分化)

### 🟢 **深刻度: 低** - 軽微な命名の不統一

- `hakorune_vm_core.hako` (アンダースコア) vs `hakorune-vm/` (ハイフン)
- `mir_vm_min.hako` (vm内) vs `mir_builder_min.hako` (shared内)

---

## 依存関係分析

### 最も使用されているBox (Top 10)

| Box | 使用回数 | 場所 | 役割 |
|-----|---------|------|------|
| `result_box.hako` | 36回 | `vm/boxes/` | Rust風エラーハンドリング |
| `string_helpers.hako` | 26回 | `shared/common/` | 文字列操作ヘルパー |
| `value_manager.hako` | 20回 | `hakorune-vm/` | VM値管理 |
| `json_field_extractor.hako` | 17回 | `hakorune-vm/` | JSON解析 |
| `string_ops.hako` | 15回 | `shared/common/` | 文字列操作 |
| `json_cursor.hako` | 11回 | `shared/json/` | JSON走査 |
| `json_scan_guard.hako` | 7回 | `hakorune-vm/` | JSONスキャン保護 |
| `reg_guard.hako` | 6回 | `hakorune-vm/` | レジスタ保護 |
| `mir_builder_min.hako` | 6回 | `shared/json/` | MIR構築 |
| `args_extractor.hako` | 3回 | `hakorune-vm/` | 引数抽出 |

### 重要な依存パターン

#### パターン1: hakorune-vm → shared の依存
```
hakorune-vm/*.hako
├── using "selfhost/vm/boxes/result_box.hako"           (36回)
├── using "selfhost/shared/common/string_helpers.hako"  (26回)
├── using "selfhost/shared/common/string_ops.hako"      (15回)
└── using "selfhost/shared/json/json_cursor.hako"       (11回)
```

**問題**: `result_box.hako` は `vm/boxes/` にあるが、実質的に全体の基盤。

#### パターン2: compiler → shared の依存
```
compiler/pipeline_v2/*.hako
├── using selfhost.common.json.mir_builder_min          (6回)
├── using selfhost.common.json.mir_v1_adapter           (1回)
└── (旧VM依存なし！)
```

**良好**: compiler は旧VM (vm/) に依存していない。

#### パターン3: 循環依存のリスク
```
shared/hako_module.toml:
[dependencies]
"selfhost.vm" = "^1.0.0"

hakorune-vm/ は shared/ を使用
shared/ は vm/ に依存
```

**問題**: `shared/common/mini_vm_*.hako` が旧VM (vm/) に依存することで循環の可能性。

---

## 推奨ディレクトリ構造

### ゴール
- **VM実装を1つに統一** (hakorune-vm を正式VMに)
- **3層アーキテクチャの明確化**: core → runtime → compiler
- **過剰な階層を削減** (4階層→3階層以下)
- **命名規則の統一** (`*_box.hako` で統一)

### 提案構造 (Phase 1-3 段階移行)

```
selfhost/
├── core/                          # 🆕 基盤レイヤー (15-20ファイル)
│   ├── hako_module.toml
│   ├── result.hako                # ← vm/boxes/result_box.hako
│   ├── string_helpers.hako        # ← shared/common/string_helpers.hako
│   ├── string_ops.hako            # ← shared/common/string_ops.hako
│   ├── json_cursor.hako           # ← shared/json/json_cursor.hako
│   ├── json_utils.hako            # ← shared/json/json_utils.hako
│   ├── json_canonical.hako        # ← shared/json/json_canonical_box.hako
│   └── [他の共通基盤]
│
├── runtime/                       # 🆕 VM実行時 (70-80ファイル)
│   ├── hako_module.toml
│   ├── vm_core.hako               # ← hakorune-vm/hakorune_vm_core.hako
│   ├── instruction_dispatcher.hako
│   ├── handlers/                  # 命令ハンドラー (22ファイル)
│   │   ├── binop_handler.hako
│   │   ├── compare_handler.hako
│   │   └── [他20種類のハンドラー]
│   ├── guards/                    # 検証・保護 (5-6ファイル)
│   │   ├── args_guard.hako
│   │   ├── reg_guard.hako
│   │   └── [他ガード]
│   ├── locators/                  # 検索・解決 (3-4ファイル)
│   │   ├── function_locator.hako
│   │   ├── blocks_locator.hako
│   │   └── instrs_locator.hako
│   ├── value_manager.hako
│   ├── error_builder.hako
│   └── tests/                     # ← hakorune-vm/tests/
│       └── [22テストファイル]
│
├── compiler/                      # 既存（変更なし）
│   ├── hako_module.toml
│   └── pipeline_v2/               # 38ファイル
│       ├── pipeline.hako
│       ├── emitters/              # 🆕 emit_*.hako をサブディレクトリに
│       │   ├── binop_emitter.hako
│       │   ├── call_emitter.hako
│       │   └── [他エミッター]
│       ├── extractors/            # 🆕 *_extract_*.hako をサブディレクトリに
│       │   ├── call_extractor.hako
│       │   └── [他抽出器]
│       └── [他パイプライン要素]
│
├── mir/                           # 🆕 MIR構築・IO (10ファイル)
│   ├── hako_module.toml
│   ├── schema.hako                # ← shared/mir/mir_schema_box.hako
│   ├── block_builder.hako         # ← shared/mir/block_builder_box.hako
│   ├── io.hako                    # ← shared/mir/mir_io_box.hako
│   ├── builder_min.hako           # ← shared/json/mir_builder_min.hako
│   ├── builder_v2.hako            # ← shared/json/mir_builder2.hako
│   ├── v1_adapter.hako            # ← shared/json/mir_v1_adapter.hako
│   └── inst_encoder.hako          # ← shared/json/json_inst_encode_box.hako
│
├── backend/                       # 🆕 バックエンドアダプター (2-3ファイル)
│   ├── hako_module.toml
│   ├── llvm_backend.hako          # ← shared/backend/llvm_backend_box.hako
│   └── host_bridge.hako           # ← shared/host_bridge/host_bridge_box.hako
│
├── tools/                         # 既存（変更なし）
│   ├── hako_module.toml
│   └── [7ファイル - dep_tree系]
│
└── tests/                         # 既存（変更なし）
    └── [4テストファイル]
```

---

## モジュール分割提案

### Module 1: `selfhost.core` (基盤)

**責任範囲**: プロジェクト全体で使用される基本ユーティリティ

**含まれるBox**:
- `result.hako` (エラーハンドリング)
- `string_helpers.hako`, `string_ops.hako` (文字列操作)
- `json_cursor.hako`, `json_utils.hako`, `json_canonical.hako` (JSON基本操作)

**依存関係**: なし (自己完結)

**exports 定義**:
```toml
[module]
name = "selfhost.core"
version = "1.0.0"

[exports]
result = "result.hako"
string_helpers = "string_helpers.hako"
string_ops = "string_ops.hako"
json.cursor = "json_cursor.hako"
json.utils = "json_utils.hako"
json.canonical = "json_canonical.hako"
```

---

### Module 2: `selfhost.runtime` (VM実行時)

**責任範囲**: MIR命令の実行、値管理、エラー処理

**含まれるBox**:
- `vm_core.hako` (VMコア)
- `instruction_dispatcher.hako` (命令ディスパッチャ)
- `handlers/*.hako` (22命令ハンドラー)
- `guards/*.hako` (検証・保護)
- `locators/*.hako` (検索・解決)
- `value_manager.hako`, `error_builder.hako`

**依存関係**: `selfhost.core`

**exports 定義**:
```toml
[module]
name = "selfhost.runtime"
version = "1.0.0"

[exports]
vm_core = "vm_core.hako"
dispatcher = "instruction_dispatcher.hako"

# 主要ハンドラー (必要に応じて公開)
handlers.binop = "handlers/binop_handler.hako"
handlers.compare = "handlers/compare_handler.hako"
# ...

[dependencies]
"selfhost.core" = "^1.0.0"
```

---

### Module 3: `selfhost.mir` (MIR構築)

**責任範囲**: MIR JSON生成、スキーマ、ブロック構築

**含まれるBox**:
- `schema.hako` (MIRスキーマ)
- `block_builder.hako` (ブロック構築)
- `io.hako` (MIR入出力)
- `builder_min.hako`, `builder_v2.hako` (MIRビルダー)
- `v1_adapter.hako` (互換性アダプター)
- `inst_encoder.hako` (命令エンコーダー)

**依存関係**: `selfhost.core`

**exports 定義**:
```toml
[module]
name = "selfhost.mir"
version = "1.0.0"

[exports]
schema = "schema.hako"
block_builder = "block_builder.hako"
io = "io.hako"
builder_min = "builder_min.hako"
builder_v2 = "builder_v2.hako"

[dependencies]
"selfhost.core" = "^1.0.0"
```

---

### Module 4: `selfhost.compiler` (既存)

**責任範囲**: Hakorune → MIR コンパイル

**変更点**:
- `emitters/` サブディレクトリに `emit_*.hako` を移動 (8ファイル)
- `extractors/` サブディレクトリに `*_extract_*.hako` を移動 (4ファイル)

**依存関係**: `selfhost.core`, `selfhost.mir`

**exports 定義** (更新):
```toml
[module]
name = "selfhost.compiler"
version = "1.0.0"

[exports]
pipeline = "pipeline_v2/pipeline.hako"
# emitters と extractors は private

[dependencies]
"selfhost.core" = "^1.0.0"
"selfhost.mir" = "^1.0.0"
```

---

### Module 5: `selfhost.backend` (バックエンド)

**責任範囲**: LLVM/ホストブリッジ等のバックエンドアダプター

**含まれるBox**:
- `llvm_backend.hako`
- `host_bridge.hako`
- `adapters/map_kv_adapter.hako`

**依存関係**: `selfhost.core`, `selfhost.mir`

**exports 定義**:
```toml
[module]
name = "selfhost.backend"
version = "1.0.0"

[exports]
llvm = "llvm_backend.hako"
host_bridge = "host_bridge.hako"

[dependencies]
"selfhost.core" = "^1.0.0"
"selfhost.mir" = "^1.0.0"
```

---

## 移動・リネーム候補

### フェーズ1: 基盤統合 (深刻度: 高)

| 現在のパス | 推奨パス | 理由 |
|-----------|---------|------|
| `vm/boxes/result_box.hako` | `core/result.hako` | 全体で使用される基盤 (36回参照) |
| `shared/common/string_helpers.hako` | `core/string_helpers.hako` | 汎用ヘルパー (26回参照) |
| `shared/common/string_ops.hako` | `core/string_ops.hako` | 汎用ヘルパー (15回参照) |
| `shared/json/json_cursor.hako` | `core/json_cursor.hako` | JSON基盤 (11回参照) |
| `shared/json/json_utils.hako` | `core/json_utils.hako` | JSON基盤 |
| `shared/json/json_canonical_box.hako` | `core/json_canonical.hako` | JSON基盤 |

**影響**: 全モジュールの `using` 文を更新 (60+箇所)

---

### フェーズ2: VM統一 (深刻度: 高)

| 現在のパス | 推奨パス | 理由 |
|-----------|---------|------|
| `hakorune-vm/hakorune_vm_core.hako` | `runtime/vm_core.hako` | VMコア |
| `hakorune-vm/instruction_dispatcher.hako` | `runtime/instruction_dispatcher.hako` | ディスパッチャ |
| `hakorune-vm/*_handler.hako` (17ファイル) | `runtime/handlers/*_handler.hako` | 命令ハンドラー群 |
| `hakorune-vm/*_guard.hako` (3ファイル) | `runtime/guards/*_guard.hako` | 検証・保護層 |
| `hakorune-vm/*_locator.hako` (3ファイル) | `runtime/locators/*_locator.hako` | 検索・解決層 |
| `hakorune-vm/value_manager.hako` | `runtime/value_manager.hako` | 値管理 |
| `hakorune-vm/error_builder.hako` | `runtime/error_builder.hako` | エラー構築 |
| `hakorune-vm/tests/` | `runtime/tests/` | VMテスト群 |

**影響**: hakorune-vm 内部の相互参照を更新 (50+箇所)

**削除対象** (旧VM):
```
vm/                          # 🗑️ 完全削除
├── boxes/mini_vm_core.hako
├── boxes/mir_vm_min.hako
└── [全30ファイル]

shared/common/               # 🗑️ 旧VM依存ファイル削除
├── mini_vm_scan.hako
├── mini_vm_binop.hako
└── mini_vm_compare.hako
```

---

### フェーズ3: MIR集約 (深刻度: 中)

| 現在のパス | 推奨パス | 理由 |
|-----------|---------|------|
| `shared/mir/mir_schema_box.hako` | `mir/schema.hako` | MIRスキーマ |
| `shared/mir/block_builder_box.hako` | `mir/block_builder.hako` | ブロック構築 |
| `shared/mir/mir_io_box.hako` | `mir/io.hako` | MIR入出力 |
| `shared/json/mir_builder_min.hako` | `mir/builder_min.hako` | MIRビルダー |
| `shared/json/mir_builder2.hako` | `mir/builder_v2.hako` | MIRビルダーv2 |
| `shared/json/mir_v1_adapter.hako` | `mir/v1_adapter.hako` | 互換性アダプター |
| `shared/json/json_inst_encode_box.hako` | `mir/inst_encoder.hako` | 命令エンコーダー |

**影響**: compiler/ からの参照を更新 (10+箇所)

---

### フェーズ4: コンパイラー整理 (深刻度: 低)

**サブディレクトリ化** (移動のみ、リネームなし):

| 現在のパス | 推奨パス | 理由 |
|-----------|---------|------|
| `compiler/pipeline_v2/emit_*.hako` (8ファイル) | `compiler/pipeline_v2/emitters/` | 機能別整理 |
| `compiler/pipeline_v2/*_extract_*.hako` (4ファイル) | `compiler/pipeline_v2/extractors/` | 機能別整理 |

**影響**: pipeline.hako からの相対パス更新のみ (12箇所)

---

## hako_module.toml 最適化

### 現状の問題

1. **shared/hako_module.toml が肥大化** (29行、exports 14個)
2. **循環依存の可能性** (`shared` → `vm`, `hakorune-vm` → `shared`)
3. **exports の命名が不統一** (`json.mir_builder_min` vs `mir.schema`)

### 最適化案

#### 1. `core/hako_module.toml` (新規)

```toml
[module]
name = "selfhost.core"
version = "1.0.0"
description = "Foundational utilities for all selfhost modules"

[exports]
# Error handling
result = "result.hako"

# String utilities
string_helpers = "string_helpers.hako"
string_ops = "string_ops.hako"

# JSON utilities
json.cursor = "json_cursor.hako"
json.utils = "json_utils.hako"
json.canonical = "json_canonical.hako"

# No dependencies - self-contained
```

---

#### 2. `runtime/hako_module.toml` (新規)

```toml
[module]
name = "selfhost.runtime"
version = "1.0.0"
description = "Hakorune VM execution runtime"

[exports]
# VM core
vm_core = "vm_core.hako"
dispatcher = "instruction_dispatcher.hako"
value_manager = "value_manager.hako"
error_builder = "error_builder.hako"

# Main handlers (公開が必要な場合のみ)
handlers.binop = "handlers/binop_handler.hako"
handlers.compare = "handlers/compare_handler.hako"
handlers.boxcall = "handlers/boxcall_handler.hako"
handlers.mircall = "handlers/mircall_handler.hako"

[private]
# Most handlers are private
# guards/* - internal validation
# locators/* - internal resolution

[dependencies]
"selfhost.core" = "^1.0.0"
```

---

#### 3. `mir/hako_module.toml` (新規)

```toml
[module]
name = "selfhost.mir"
version = "1.0.0"
description = "MIR construction and I/O utilities"

[exports]
# Schema and structure
schema = "schema.hako"
block_builder = "block_builder.hako"
io = "io.hako"

# Builders
builder_min = "builder_min.hako"
builder_v2 = "builder_v2.hako"

# Adapters
v1_adapter = "v1_adapter.hako"
inst_encoder = "inst_encoder.hako"

[dependencies]
"selfhost.core" = "^1.0.0"
```

---

#### 4. `compiler/hako_module.toml` (更新)

```toml
[module]
name = "selfhost.compiler"
version = "1.0.0"
description = "Hakorune to MIR compiler pipeline"

[exports]
# Main entry point
pipeline = "pipeline_v2/pipeline.hako"

# Publicly accessible pipeline stages (if needed)
# stages.normalize = "pipeline_v2/normalizer_box.hako"
# stages.ssa = "pipeline_v2/local_ssa_box.hako"

[private]
# emitters/* - internal code generation
# extractors/* - internal AST processing
# All other pipeline_v2/* files

[dependencies]
"selfhost.core" = "^1.0.0"
"selfhost.mir" = "^1.0.0"
```

---

#### 5. `backend/hako_module.toml` (新規)

```toml
[module]
name = "selfhost.backend"
version = "1.0.0"
description = "Backend adapters (LLVM, host bridge, etc.)"

[exports]
llvm = "llvm_backend.hako"
host_bridge = "host_bridge.hako"

[dependencies]
"selfhost.core" = "^1.0.0"
"selfhost.mir" = "^1.0.0"
```

---

#### 6. `tools/hako_module.toml` (既存・変更なし)

```toml
[module]
name = "selfhost.tools"
version = "1.0.0"
description = "Development tools (dependency analysis, etc.)"

[exports]
dep_tree = "dep_tree.hako"
dep_tree_main = "dep_tree_main.hako"

[dependencies]
"selfhost.core" = "^1.0.0"
```

---

## マイグレーション計画

### Phase 1: 基盤統合 (Week 1-2) ⭐最優先

**目標**: `core/` モジュール確立、循環依存解消

**タスク**:
1. ✅ `core/` ディレクトリ作成
2. ✅ 基盤ファイル移動 (6ファイル):
   - `vm/boxes/result_box.hako` → `core/result.hako`
   - `shared/common/string_*.hako` → `core/`
   - `shared/json/json_*.hako` → `core/`
3. ✅ `core/hako_module.toml` 作成
4. ✅ 全モジュールの `using` 文を更新 (60+箇所)
   - 検索: `grep -r "using.*result_box" selfhost/`
   - 置換: `"selfhost/vm/boxes/result_box.hako"` → `selfhost.core.result`
5. ✅ スモークテスト実行・検証

**リスク**:
- **高**: 全モジュールに影響 (60+ファイル変更)
- **緩和策**: 段階的コミット、各ファイル移動後に即テスト

**成功基準**:
- [ ] 既存の全テストがPASS
- [ ] `using` 構文が統一 (`selfhost.core.*`)
- [ ] 循環依存がない (`core` は他に依存しない)

---

### Phase 2: VM統一 (Week 3-4)

**目標**: `runtime/` モジュール確立、旧VM (`vm/`) 削除

**タスク**:
1. ✅ `runtime/` ディレクトリ作成
2. ✅ hakorune-vm ファイル移動 (60+ファイル):
   - `hakorune-vm/*.hako` → `runtime/`
   - `hakorune-vm/tests/` → `runtime/tests/`
3. ✅ サブディレクトリ整理:
   - `*_handler.hako` → `runtime/handlers/`
   - `*_guard.hako` → `runtime/guards/`
   - `*_locator.hako` → `runtime/locators/`
4. ✅ `runtime/hako_module.toml` 作成
5. ✅ 内部参照を更新 (50+箇所)
6. ✅ 旧VM削除:
   - `vm/` 全体 (30ファイル)
   - `shared/common/mini_vm_*.hako` (3ファイル)
7. ✅ スモークテスト実行・検証

**リスク**:
- **中**: hakorune-vm 内部の相互参照が複雑
- **緩和策**: ディレクトリ構造のみ変更 → テスト → リネーム

**成功基準**:
- [ ] runtime/tests/ の全22テストがPASS
- [ ] 旧VM (vm/) が完全削除
- [ ] `using selfhost.runtime.*` で統一

---

### Phase 3: MIR集約 (Week 5)

**目標**: `mir/` モジュール確立、MIR関連ファイルの集約

**タスク**:
1. ✅ `mir/` ディレクトリ作成
2. ✅ MIRファイル移動 (7ファイル):
   - `shared/mir/*.hako` → `mir/`
   - `shared/json/mir_*.hako` → `mir/`
   - `shared/json/json_inst_encode_box.hako` → `mir/inst_encoder.hako`
3. ✅ `mir/hako_module.toml` 作成
4. ✅ compiler/ からの参照を更新 (10+箇所)
5. ✅ スモークテスト実行・検証

**リスク**:
- **低**: compiler/ からの参照のみ更新
- **緩和策**: compiler/ の単体テスト先行

**成功基準**:
- [ ] compiler/ の全テストがPASS
- [ ] `using selfhost.mir.*` で統一

---

### Phase 4: コンパイラー整理 (Week 6)

**目標**: `compiler/pipeline_v2/` サブディレクトリ整理

**タスク**:
1. ✅ サブディレクトリ作成:
   - `compiler/pipeline_v2/emitters/`
   - `compiler/pipeline_v2/extractors/`
2. ✅ ファイル移動 (移動のみ、リネームなし):
   - `emit_*.hako` → `emitters/`
   - `*_extract_*.hako` → `extractors/`
3. ✅ `pipeline.hako` の相対パス更新 (12箇所)
4. ✅ スモークテスト実行・検証

**リスク**:
- **極低**: 相対パス更新のみ
- **緩和策**: 先に git mv → テスト → コミット

**成功基準**:
- [ ] compiler/ の全テストがPASS
- [ ] ディレクトリ構造が見やすくなる

---

### Phase 5: Backend分離 (Week 7、オプショナル)

**目標**: `backend/` モジュール確立

**タスク**:
1. ✅ `backend/` ディレクトリ作成
2. ✅ バックエンドファイル移動 (3ファイル):
   - `shared/backend/llvm_backend_box.hako` → `backend/llvm_backend.hako`
   - `shared/host_bridge/host_bridge_box.hako` → `backend/host_bridge.hako`
   - `shared/adapters/map_kv_*.hako` → `backend/adapters/`
3. ✅ `backend/hako_module.toml` 作成
4. ✅ スモークテスト実行・検証

**リスク**:
- **極低**: 影響範囲が限定的
- **緩和策**: 必要に応じてスキップ可

**成功基準**:
- [ ] 全テストがPASS
- [ ] `using selfhost.backend.*` で統一

---

## 命名規則の統一提案

### 現状の混乱

| パターン | 使用箇所 | 例 |
|---------|---------|-----|
| `*_box.hako` | compiler, shared, vm | `mir_builder_box.hako` |
| `*_handler.hako` | hakorune-vm | `binop_handler.hako` |
| `*_guard.hako` | hakorune-vm | `args_guard.hako` |
| `*.hako` (サフィックスなし) | 散在 | `pipeline.hako` |

### 推奨命名規則

#### ルール1: Box実装は `_box` サフィックスを省略
```
❌ 旧: mir_builder_box.hako
✅ 新: mir_builder.hako

❌ 旧: string_helpers_box.hako
✅ 新: string_helpers.hako
```

**理由**: Hakoruneでは「Everything is Box」なので冗長。

#### ルール2: 役割別サフィックスを維持
```
✅ handlers/binop_handler.hako     # 命令ハンドラー
✅ guards/args_guard.hako           # 検証・保護
✅ locators/function_locator.hako   # 検索・解決
✅ extractors/call_extractor.hako   # AST抽出
✅ emitters/binop_emitter.hako      # コード生成
```

**理由**: 役割を明示、サブディレクトリと組み合わせて理解しやすい。

#### ルール3: ファイル名は snake_case
```
✅ vm_core.hako
✅ instruction_dispatcher.hako
❌ hakoruneVmCore.hako (camelCase禁止)
❌ InstructionDispatcher.hako (PascalCase禁止)
```

**理由**: Rust/Python慣例に従う。

---

## 実装優先度マトリックス

| Phase | 深刻度 | 影響範囲 | 工数 | 優先度 |
|-------|-------|---------|------|--------|
| **Phase 1: 基盤統合** | 🔴 高 | 全体 (60+ファイル) | 2週間 | ⭐⭐⭐ 最優先 |
| **Phase 2: VM統一** | 🔴 高 | runtime (60+ファイル) | 2週間 | ⭐⭐⭐ 最優先 |
| **Phase 3: MIR集約** | 🟡 中 | compiler (10+ファイル) | 1週間 | ⭐⭐ 高 |
| **Phase 4: Compiler整理** | 🟢 低 | compiler (12箇所) | 1週間 | ⭐ 中 |
| **Phase 5: Backend分離** | 🟢 低 | backend (3ファイル) | 1週間 | - オプショナル |

**総工数**: 5-7週間
**最小限 (Phase 1-3のみ)**: 5週間

---

## リスク分析と緩和策

### リスク1: 大規模移動による一時的な不安定化 (🔴 高)

**影響**: Phase 1-2で100+ファイルが影響

**緩和策**:
1. ✅ **段階的コミット**: 1ファイル移動 → テスト → コミット
2. ✅ **ロールバック可能**: 各Phase後にgitタグ作成
3. ✅ **CI/CDゲート**: 全スモークテストPASSまでマージ禁止
4. ✅ **ブランチ戦略**: `feature/module-reorg-phase1` など分離

---

### リスク2: using 構文の更新漏れ (🟡 中)

**影響**: 60+箇所の `using` 文を手動更新

**緩和策**:
1. ✅ **スクリプト支援**: `sed` / `find-replace` で一括置換
   ```bash
   find selfhost/ -name "*.hako" -exec sed -i \
     's|"selfhost/vm/boxes/result_box.hako"|selfhost.core.result|g' {} +
   ```
2. ✅ **コンパイラー検証**: Rust側のパーサーが未解決参照を検出
3. ✅ **Grep検証**: 移動前に `grep -r "old_path" selfhost/` で完全確認

---

### リスク3: テストの一時的な失敗 (🟡 中)

**影響**: Phase 1-2で一部テストが失敗する可能性

**緩和策**:
1. ✅ **最小構成テスト**: 各移動後に `result.hako` 単体テスト
2. ✅ **段階的統合**: core → runtime → mir の順で統合
3. ✅ **デバッグモード**: `NYASH_CLI_VERBOSE=1` で詳細ログ

---

### リスク4: ドキュメントの更新漏れ (🟢 低)

**影響**: README/ドキュメントが古いパスを参照

**緩和策**:
1. ✅ **ドキュメント検索**: `grep -r "selfhost/" docs/`
2. ✅ **Migration Guide作成**: Phase完了後に更新ガイド公開
3. ✅ **Deprecation Notice**: 旧パスに移動先を明記

---

## 成功基準

### Phase 1完了 (基盤統合)
- [ ] `core/` モジュールが確立
- [ ] `result.hako`, `string_helpers.hako` など6ファイルが移動
- [ ] 全モジュールが `selfhost.core.*` を使用
- [ ] 既存の全テストがPASS
- [ ] 循環依存がない

### Phase 2完了 (VM統一)
- [ ] `runtime/` モジュールが確立
- [ ] hakorune-vm の60+ファイルが移動
- [ ] 旧VM (`vm/`) が完全削除
- [ ] runtime/tests/ の全22テストがPASS
- [ ] `using selfhost.runtime.*` で統一

### Phase 3完了 (MIR集約)
- [ ] `mir/` モジュールが確立
- [ ] MIR関連7ファイルが移動
- [ ] compiler/ が `selfhost.mir.*` を使用
- [ ] 全テストがPASS

### Phase 4完了 (Compiler整理)
- [ ] `emitters/`, `extractors/` サブディレクトリ確立
- [ ] 12ファイルが移動
- [ ] 全テストがPASS

### 最終成功基準 (全Phase完了)
- [ ] ディレクトリ構造が3層以下 (最大: `runtime/handlers/`)
- [ ] 命名規則が統一 (`*_box` サフィックス削除、役割別サフィックス維持)
- [ ] モジュール境界が明確 (core/runtime/mir/compiler/backend)
- [ ] 循環依存がない
- [ ] 全165ファイルが適切な場所に配置
- [ ] 全テストがPASS (170+ PASS)
- [ ] ドキュメントが更新

---

## 次のアクション

### 即座に実施 (Week 0)
1. ✅ このレポートをレビュー・承認
2. ✅ `feature/module-reorg-phase1` ブランチ作成
3. ✅ 移動スクリプト準備 (`tools/migrate_phase1.sh`)

### Week 1-2 (Phase 1)
1. ✅ `core/` ディレクトリ作成
2. ✅ 基盤ファイル移動 (6ファイル)
3. ✅ `core/hako_module.toml` 作成
4. ✅ 全 `using` 文を更新 (60+箇所)
5. ✅ スモークテスト実行・検証
6. ✅ Phase 1 完了タグ作成 (`v1.0-phase1-core`)

### Week 3-4 (Phase 2)
1. ✅ `runtime/` ディレクトリ作成
2. ✅ hakorune-vm ファイル移動 (60+ファイル)
3. ✅ サブディレクトリ整理
4. ✅ 旧VM削除
5. ✅ スモークテスト実行・検証
6. ✅ Phase 2 完了タグ作成 (`v1.0-phase2-runtime`)

### Week 5 (Phase 3)
1. ✅ `mir/` ディレクトリ作成
2. ✅ MIRファイル移動 (7ファイル)
3. ✅ スモークテスト実行・検証
4. ✅ Phase 3 完了タグ作成 (`v1.0-phase3-mir`)

---

## 参考資料

### 依存関係グラフ (提案後)

```
selfhost.tools ────────┐
                       │
selfhost.backend ──────┼─────┐
                       │     │
selfhost.compiler ─────┼────┐│
                       │    ││
selfhost.runtime ──────┼───┐││
                       │   │││
selfhost.mir ──────────┼──┐│││
                       │  ││││
selfhost.core ─────────┴──┴┴┴┴  (基盤・依存なし)
```

**循環依存なし**: すべてのモジュールが `core` のみに依存、または core + 1レイヤーに依存。

---

## まとめ

### 現状の課題
- 🔴 **2つのVM実装が並存** (vm/ と hakorune-vm/)
- 🔴 **命名の不統一** (`*_box`, `*_handler`, サフィックスなし混在)
- 🟡 **過剰な階層** (4階層、空ディレクトリ存在)
- 🟡 **モジュール境界不明確** (shared/ が肥大化)

### 提案の効果
- ✅ **VM統一** (runtime/ に一本化、旧VM削除)
- ✅ **3層アーキテクチャ** (core → runtime → compiler)
- ✅ **命名規則統一** (`*_box` 削減、役割別サフィックス維持)
- ✅ **循環依存解消** (core が基盤、他は依存)
- ✅ **テスト集約** (runtime/tests/ に22テスト集約)

### 実装計画
- ⏱️ **総工数**: 5-7週間
- 🎯 **最小限 (Phase 1-3)**: 5週間
- ⭐ **最優先**: Phase 1 (基盤統合) + Phase 2 (VM統一)
- 🛡️ **リスク緩和**: 段階的コミット、ロールバック可能

---

**End of Report**
