# 依存関係グラフ（テキスト形式）

## レイヤー間依存関係

```
┌─────────────────────────────────────┐
│  Layer 5: Tools / Tests             │
│  - tools/*  (4 modules)             │
│  - tests/*  (included in vm layer)  │
└─────────────────────────────────────┘
           ↓ [1 dependency]
┌─────────────────────────────────────┐
│  Layer 4: Compiler                  │  32 modules
│  - compiler/pipeline_v2/*           │
│                                     │
│  Hub: regex_flow (15 in)            │
│  Complex: pipeline (25 out)         │
└─────────────────────────────────────┘
   ↓ [5 deps]              ↓ [2 deps]
   ↓ (to shared_mir)       ↓ (to shared_json)
   ↓                       ↓
┌──┴──────────────────────┴──────────┐
│  Layer 3: VM                       │  84 modules
│  - hakorune-vm/*  (60 modules)     │
│  - vm/*           (24 modules)     │
│                                    │
│  🔴 Hubs:                          │
│    - hakorune_vm_core (16 in/17 out)│
│    - value_manager (20 in)         │
│    - json_field_extractor (17 in)  │
│    - instruction_dispatcher (19 out)│
│                                    │
│  ⚠️ Violations:                    │
│    - vm.flow_runner → compiler ❌  │
└────────────────────────────────────┘
   ↓ [13 deps]      ↓ [12 deps]
   ↓ (hakorune_vm)  ↓ (vm)
   ↓                ↓
┌──┴────────────────┴────────────────┐
│  Layer 2: Infrastructure           │  11 modules
│  - shared/json/*  (5 modules)      │
│  - shared/mir/*   (3 modules)      │
│  - shared/backend/* (1 module)     │
│                                    │
│  🔴 Hubs:                          │
│    - json_cursor (22 in)           │
│    - json_frag (6 in)              │
│    - block_builder_box (5 in)      │
│                                    │
│  ⚠️ Violations:                    │
│    - shared.mir.mir_io_box → hakorune_vm (6 deps) ❌
│    - shared.json_adapter → vm.json_cur ❌
└────────────────────────────────────┘
           ↓ [1 dependency]
┌─────────────────────────────────────┐
│  Layer 1: Foundation                │  3 modules
│  - shared/common/*                  │
│    - string_helpers                 │
│    - string_ops                     │
│    - mini_vm_* ⚠️ (misplaced)       │
│                                     │
│  ⚠️ Violations:                     │
│    - mini_vm_binop → vm.json ❌     │
│    - mini_vm_compare → vm.scan ❌   │
│    - mini_vm_scan → shared.json ❌  │
└─────────────────────────────────────┘
```

---

## Top 19 主要モジュール依存関係

### Compiler Layer
```
compiler.pipeline_v2.pipeline (25 out, 1 in)
  ├→ stage1_extract_flow
  ├→ stage1_json_scanner_box
  ├→ new_extract_box
  ├→ method_extract_box
  ├→ normalizer_box
  ├→ name_resolve_box
  ├→ local_ssa_box
  ├→ pipeline_emit_box
  ├→ emit_mir_flow
  ├→ emit_mir_flow_map
  └→ ... (15 more)

compiler.pipeline_v2.regex_flow (1 out, 15 in) 🔴 Hub
  ← stage1_extract_flow
  ← stage1_json_scanner_box
  ← new_extract_box
  ← method_extract_box
  ← normalizer_box
  └← ... (10 more dependents)
```

### VM Layer - Hakorune VM
```
hakorune-vm.hakorune_vm_core (17 out, 16 in) 🔴 Hub
  ├→ value_manager
  ├→ json_field_extractor
  ├→ instruction_dispatcher
  ├→ block_mapper
  ├→ json_scan_guard
  └→ ... (12 more)
  ←─ boxcall_handler
  ←─ compare_handler
  ←─ binop_handler
  └← ... (13 more dependents)

hakorune-vm.value_manager (2 out, 20 in) 🔴 Hub
  ├→ reg_guard
  ├→ shared.json.json_cursor
  └← ... (20 dependents)

hakorune-vm.json_field_extractor (2 out, 17 in) 🔴 Hub
  ├→ json_scan_guard
  ├→ shared.json.json_cursor
  └← ... (17 dependents)

hakorune-vm.instruction_dispatcher (19 out, 1 in) 🔴 Complex
  ├→ binop_handler
  ├→ compare_handler
  ├→ boxcall_handler
  ├→ const_handler
  ├→ copy_handler
  ├→ extern_call_handler
  ├→ load_handler
  ├→ method_call_handler
  ├→ mircall_handler
  ├→ newbox_handler
  ├→ phi_handler
  ├→ store_handler
  ├→ unaryop_handler
  └→ ... (6 more handlers)

hakorune-vm.mircall_handler (17 out, 1 in) 🔴 Complex
  ├→ hakorune_vm_core
  ├→ value_manager
  ├→ json_field_extractor
  ├→ callee_parser
  ├→ args_extractor
  └→ ... (12 more)
```

### VM Layer - Mini VM
```
vm.boxes.mir_vm_min (14 out, 1 in) 🔴 Complex
  ├→ hakorune_vm_core
  ├→ value_manager
  ├→ json_field_extractor
  ├→ function_locator
  ├→ blocks_locator
  └→ ... (9 more)

vm.flow_runner (3 out, 0 in) ⚠️ Violation
  ├→ compiler.pipeline_v2.flow_entry ❌ Layer violation
  ├→ shared.json.json_cursor
  └→ shared.common.string_ops
```

### Infrastructure Layer
```
shared.json.json_cursor (3 out, 22 in) 🔴 Hub
  ├→ json_frag
  ├→ string_scan
  └→ string_helpers
  ←─ hakorune-vm.value_manager
  ←─ hakorune-vm.json_field_extractor
  ←─ hakorune-vm.json_scan_guard
  ←─ hakorune-vm.function_locator
  ←─ hakorune-vm.blocks_locator
  └← ... (17 more dependents)

shared.json.utils.json_frag (2 out, 6 in) 🟡 Hub
  ├→ string_scan
  ├→ string_helpers
  └← ... (6 dependents)

shared.mir.block_builder_box (1 out, 5 in) 🟡 Hub
  ├→ mir_io_box
  └← ... (5 dependents)

shared.mir.mir_io_box (9 out, 0 in) ⚠️ Complex + Violations
  ├→ hakorune-vm.function_locator ❌ Layer violation
  ├→ hakorune-vm.blocks_locator ❌
  ├→ hakorune-vm.instrs_locator ❌
  ├→ hakorune-vm.block_iterator ❌
  ├→ hakorune-vm.backward_object_scanner ❌
  ├→ vm.boxes.result_box ❌
  └→ ... (3 more VM dependencies)
```

### Foundation Layer
```
shared.common.string_helpers (0 out, 68 in) 🔴 Super Hub
  ← ... (68 dependents across all layers)

shared.common.string_ops (0 out, 22 in) 🔴 Hub
  ← ... (22 dependents)

shared.common.mini_vm_binop (2 out, 0 in) ⚠️ Misplaced
  ├→ vm.json ❌ Layer violation (Severity 2)
  └→ vm.scan ❌ Layer violation (Severity 2)

shared.common.mini_vm_compare (1 out, 0 in) ⚠️ Misplaced
  └→ vm.scan ❌ Layer violation (Severity 2)

shared.common.mini_vm_scan (1 out, 0 in) ⚠️ Misplaced
  └→ shared.json.json_cursor ❌ Layer violation (Severity 1)
```

---

## 依存関係の健全性指標

### ✅ Good Patterns
```
1. 一方向依存: A → B → C （循環なし）
2. 安定依存: 不安定なモジュールが安定なモジュールに依存
3. レイヤー遵守: 上位 → 下位のみ
```

### ⚠️ Code Smells
```
1. Hub modules (15+ dependents):
   - string_helpers (68) ← 極端に多い
   - json_cursor (22)
   - value_manager (20)

2. Complex modules (15+ dependencies):
   - pipeline (25)
   - instruction_dispatcher (19)
   - mircall_handler (17)
   - hakorune_vm_core (17)

3. Layer violations (12):
   - Foundation → VM (Severity 2) × 3
   - Infrastructure → VM (Severity 1) × 8
   - VM → Compiler (Severity 1) × 1
```

### ❌ Anti-Patterns
```
現在は検出されず ✅
- 循環依存: 0
- 双方向依存: 0
```

---

## 依存関係マトリックス（主要モジュール）

```
                     ┌─Compiler─┬──VM──┬─Infra─┬─Found─┐
compiler.pipeline    │    ✓     │  ✓   │   ✓   │   ✓   │ 25
compiler.regex_flow  │    -     │  -   │   ✓   │   -   │  1
hakorune_vm_core     │    -     │  ✓   │   ✓   │   ✓   │ 17
value_manager        │    -     │  ✓   │   ✓   │   -   │  2
json_field_extractor │    -     │  -   │   ✓   │   -   │  2
instruction_dispatch │    -     │  ✓   │   ✓   │   ✓   │ 19
mircall_handler      │    -     │  ✓   │   ✓   │   ✓   │ 17
mir_vm_min           │    -     │  ✓   │   ✓   │   ✓   │ 14
json_cursor          │    -     │  -   │   ✓   │   ✓   │  3
mir_io_box           │    -     │  ✓❌ │   ✓   │   -   │  9
flow_runner          │    ✓❌   │  -   │   ✓   │   ✓   │  3
mini_vm_binop        │    -     │  ✓❌ │   -   │   -   │  2
mini_vm_compare      │    -     │  ✓❌ │   -   │   -   │  1
└──────────────────────────────────────────────────────┘
Legend:
  ✓  = Valid dependency
  ✓❌ = Layer violation
  -  = No dependency
  Number = Total efferent coupling
```

---

## 疎結合化の機会

### 優先度: High
1. **Extract Interface**
   - `json_cursor` (22 dependents)
   - `value_manager` (20 dependents)
   - `json_field_extractor` (17 dependents)

2. **Move to correct layer**
   - `mini_vm_*` → `vm/mini/`
   - `result_box` → `shared/common/`

### 優先度: Medium
3. **Split complex modules**
   - `pipeline` (25 deps) → 4 modules
   - `instruction_dispatcher` (19 deps) → Facade
   - `hakorune_vm_core` (17 deps) → 3 modules

4. **Introduce Facade**
   - `HakoruneVmFacade` (aggregate vm modules)
   - `CompilerPipelineFacade` (aggregate compiler modules)

### 優先度: Low
5. **Remove unused imports** (86 imports across 36 files)

---

## 依存関係の理想形

```
理想的な依存関係の方向:

Tools/Tests
    ↓ (uses)
Compiler ──────────┐
    ↓              ↓ (facade)
VM ────────────────┤
    ↓              ↓ (facade)
Infrastructure ────┤
    ↓              ↓ (facade)
Foundation ────────┘
    ↓
  (none)

各レイヤーはFacadeを通じて下位レイヤーにアクセス
直接依存は最小限に抑える
```

---

**生成日**: 2025-10-15
**分析対象**: selfhost/ (165 modules, 447 dependencies)
