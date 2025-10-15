# モジュール依存関係図

**作成日**: 2025-10-15

---

## 現状の依存関係 (混乱状態)

```
┌─────────────────┐
│   compiler/     │
│  pipeline_v2/   │
└────────┬────────┘
         │ using selfhost.common.json.*
         │ using selfhost.vm.* (一部)
         ▼
┌─────────────────┐         ┌─────────────────┐
│    shared/      │◄────────│  hakorune-vm/   │
│  (23ファイル)    │  using  │  (67ファイル)    │
└────────┬────────┘         └─────────────────┘
         │
         │ [dependencies]
         │ "selfhost.vm" = "^1.0.0"
         ▼
┌─────────────────┐
│      vm/        │
│  (30ファイル)    │
│   旧Mini-VM     │
└─────────────────┘

⚠️ 問題:
1. 循環依存のリスク (shared ←→ hakorune-vm, vm)
2. VM実装が2つ (vm/ と hakorune-vm/)
3. shared/ が肥大化 (役割不明確)
```

---

## 提案後の依存関係 (クリーンな階層)

### 全体図

```
┌─────────────────┐
│  selfhost.tools │  # 開発ツール (dep_tree等)
└────────┬────────┘
         │
         ├──────────┐
         │          │
         ▼          ▼
┌─────────────────┐ ┌─────────────────┐
│selfhost.backend │ │selfhost.compiler│  # 上位レイヤー
│  (LLVM等)       │ │  (pipeline_v2)  │
└────────┬────────┘ └────────┬────────┘
         │                   │
         │                   │
         └─────────┬─────────┘
                   │
         ┌─────────┴─────────┐
         │                   │
         ▼                   ▼
┌─────────────────┐ ┌─────────────────┐
│ selfhost.mir    │ │ selfhost.runtime│  # 中位レイヤー
│  (MIR構築)      │ │  (VM実行)       │
└────────┬────────┘ └────────┬────────┘
         │                   │
         └─────────┬─────────┘
                   │
                   ▼
         ┌─────────────────┐
         │  selfhost.core  │  # 基盤レイヤー
         │  (共通基盤)      │
         │  依存なし        │
         └─────────────────┘

✅ 利点:
1. 循環依存なし (すべてが core を基盤に)
2. 明確な階層構造 (3層: 基盤 → 中位 → 上位)
3. VM統一 (runtime/ のみ)
```

---

## レイヤー別詳細

### Layer 0: 基盤レイヤー (selfhost.core)

**役割**: プロジェクト全体の基本ユーティリティ

```
┌───────────────────────────────────────────┐
│         selfhost.core                     │
├───────────────────────────────────────────┤
│ • result.hako (エラーハンドリング)        │
│ • string_helpers.hako (文字列操作)        │
│ • string_ops.hako (文字列操作)            │
│ • json_cursor.hako (JSON走査)             │
│ • json_utils.hako (JSON操作)              │
│ • json_canonical.hako (JSON正規化)        │
├───────────────────────────────────────────┤
│ 依存: なし (自己完結)                     │
│ 使用者: 全モジュール                      │
└───────────────────────────────────────────┘

最も重要な依存:
• result.hako: 36箇所で使用
• string_helpers.hako: 26箇所で使用
• json_cursor.hako: 11箇所で使用
```

---

### Layer 1: 中位レイヤー (runtime + mir)

#### selfhost.runtime (VM実行時)

```
┌───────────────────────────────────────────┐
│       selfhost.runtime                    │
├───────────────────────────────────────────┤
│ • vm_core.hako (VMコア)                   │
│ • instruction_dispatcher.hako             │
│                                           │
│ handlers/ (22命令ハンドラー)              │
│  ├─ binop_handler.hako                    │
│  ├─ compare_handler.hako                  │
│  ├─ boxcall_handler.hako                  │
│  └─ [他19種類]                            │
│                                           │
│ guards/ (検証・保護)                      │
│  ├─ args_guard.hako                       │
│  ├─ reg_guard.hako                        │
│  └─ receiver_guard.hako                   │
│                                           │
│ locators/ (検索・解決)                    │
│  ├─ function_locator.hako                 │
│  ├─ blocks_locator.hako                   │
│  └─ instrs_locator.hako                   │
│                                           │
│ • value_manager.hako                      │
│ • error_builder.hako                      │
│                                           │
│ tests/ (22テストファイル)                 │
├───────────────────────────────────────────┤
│ 依存: selfhost.core                       │
│ 使用者: selfhost.compiler (実行時)        │
└───────────────────────────────────────────┘
```

#### selfhost.mir (MIR構築)

```
┌───────────────────────────────────────────┐
│         selfhost.mir                      │
├───────────────────────────────────────────┤
│ • schema.hako (MIRスキーマ)               │
│ • block_builder.hako (ブロック構築)       │
│ • io.hako (MIR入出力)                     │
│ • builder_min.hako (MIRビルダー)          │
│ • builder_v2.hako (MIRビルダーv2)         │
│ • v1_adapter.hako (互換性アダプター)      │
│ • inst_encoder.hako (命令エンコーダー)    │
├───────────────────────────────────────────┤
│ 依存: selfhost.core                       │
│ 使用者: selfhost.compiler                 │
└───────────────────────────────────────────┘
```

---

### Layer 2: 上位レイヤー (compiler + backend)

#### selfhost.compiler

```
┌───────────────────────────────────────────┐
│       selfhost.compiler                   │
├───────────────────────────────────────────┤
│ pipeline_v2/                              │
│  ├─ pipeline.hako (メインパイプライン)    │
│  ├─ normalizer_box.hako                   │
│  ├─ local_ssa_box.hako                    │
│  │                                        │
│  ├─ emitters/ (コード生成)                │
│  │   ├─ binop_emitter.hako               │
│  │   ├─ call_emitter.hako                │
│  │   └─ [他6種類]                         │
│  │                                        │
│  └─ extractors/ (AST抽出)                 │
│      ├─ call_extractor.hako              │
│      └─ [他3種類]                         │
├───────────────────────────────────────────┤
│ 依存: selfhost.core, selfhost.mir         │
│ 使用者: エンドユーザー (CLI)               │
└───────────────────────────────────────────┘
```

#### selfhost.backend

```
┌───────────────────────────────────────────┐
│        selfhost.backend                   │
├───────────────────────────────────────────┤
│ • llvm_backend.hako (LLVM接続)            │
│ • host_bridge.hako (ホストAPI)            │
│ • adapters/ (アダプター層)                │
├───────────────────────────────────────────┤
│ 依存: selfhost.core, selfhost.mir         │
│ 使用者: selfhost.compiler, selfhost.runtime│
└───────────────────────────────────────────┘
```

---

## 依存関係マトリックス

| モジュール | core | runtime | mir | compiler | backend | tools |
|-----------|------|---------|-----|----------|---------|-------|
| **core** | - | - | - | - | - | - |
| **runtime** | ✅ | - | - | - | - | - |
| **mir** | ✅ | - | - | - | - | - |
| **compiler** | ✅ | - | ✅ | - | - | - |
| **backend** | ✅ | - | ✅ | - | - | - |
| **tools** | ✅ | - | - | - | - | - |

**凡例**:
- ✅ : 依存あり
- - : 依存なし

**重要**: すべてのモジュールが `core` のみに依存、または `core` + 1レイヤーに依存。循環依存なし。

---

## ファイル移動マップ

### Phase 1: 基盤統合

```
vm/boxes/result_box.hako
  └─→ core/result.hako

shared/common/string_helpers.hako
  └─→ core/string_helpers.hako

shared/common/string_ops.hako
  └─→ core/string_ops.hako

shared/json/json_cursor.hako
  └─→ core/json_cursor.hako

shared/json/json_utils.hako
  └─→ core/json_utils.hako

shared/json/json_canonical_box.hako
  └─→ core/json_canonical.hako
```

---

### Phase 2: VM統一

```
hakorune-vm/*.hako (44ファイル)
  └─→ runtime/*.hako

hakorune-vm/tests/*.hako (22ファイル)
  └─→ runtime/tests/*.hako

サブディレクトリ整理:
  *_handler.hako → runtime/handlers/
  *_guard.hako → runtime/guards/
  *_locator.hako → runtime/locators/

削除:
  vm/ (全30ファイル)
  shared/common/mini_vm_*.hako (3ファイル)
```

---

### Phase 3: MIR集約

```
shared/mir/mir_schema_box.hako
  └─→ mir/schema.hako

shared/mir/block_builder_box.hako
  └─→ mir/block_builder.hako

shared/mir/mir_io_box.hako
  └─→ mir/io.hako

shared/json/mir_builder_min.hako
  └─→ mir/builder_min.hako

shared/json/mir_builder2.hako
  └─→ mir/builder_v2.hako

shared/json/mir_v1_adapter.hako
  └─→ mir/v1_adapter.hako

shared/json/json_inst_encode_box.hako
  └─→ mir/inst_encoder.hako
```

---

## 依存解決の流れ (実行時)

### コンパイル時 (Hakorune → MIR)

```
1. エンドユーザー
   │
   ▼
2. selfhost.compiler
   │ ├─→ selfhost.core (基盤ユーティリティ)
   │ └─→ selfhost.mir (MIR構築)
   │
   ▼
3. MIR JSON生成
```

### 実行時 (MIR → 実行)

```
1. MIR JSON
   │
   ▼
2. selfhost.runtime
   │ ├─→ selfhost.core (基盤ユーティリティ)
   │ └─→ instruction_dispatcher
   │      ├─→ handlers/* (22命令ハンドラー)
   │      ├─→ guards/* (検証・保護)
   │      └─→ locators/* (検索・解決)
   │
   ▼
3. 実行結果
```

---

## 循環依存の検証

### チェック方法

```bash
# Phase 1完了後に実行
cd selfhost/
for module in core runtime mir compiler backend tools; do
  echo "=== Checking $module ==="
  find $module -name "*.hako" -exec grep -h "^using" {} \; | sort | uniq
done
```

### 期待される結果 (Phase 3完了後)

```
=== core ===
(依存なし)

=== runtime ===
using selfhost.core.*

=== mir ===
using selfhost.core.*

=== compiler ===
using selfhost.core.*
using selfhost.mir.*

=== backend ===
using selfhost.core.*
using selfhost.mir.*

=== tools ===
using selfhost.core.*
```

**✅ 循環なし**: すべてが `core` を基盤に構築。

---

## まとめ

### ビフォー (現状)

```
❌ 複雑な依存関係 (shared ←→ hakorune-vm, vm)
❌ VM実装2つ (vm/ と hakorune-vm/)
❌ 循環依存のリスク
```

### アフター (Phase 3完了後)

```
✅ 明確な3層アーキテクチャ
   基盤 (core) → 中位 (runtime, mir) → 上位 (compiler, backend)

✅ 循環依存なし (すべてが core を基盤に)

✅ VM統一 (runtime/ のみ)
```

---

**End of Document**
