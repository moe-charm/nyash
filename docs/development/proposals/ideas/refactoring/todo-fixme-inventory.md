# TODO/FIXME Inventory (2025-10-04)

## Overview

調査日: 2025-10-04
調査者: Claude Code
対象: Rust codebase の TODO/FIXME コメント

**Total count**: 45 items

## Categorization

### 1. Code Generation (bid-codegen-from-copilot) - 10 items

**Status**: 🔧 **Future Implementation** - Low priority

未実装のコードジェネレーションターゲット：

**LLVM Target** (1):
- `src/bid-codegen-from-copilot/codegen/targets/llvm.rs:13` - Implement LLVM code generation

**TypeScript Target** (1):
- `src/bid-codegen-from-copilot/codegen/targets/typescript.rs:13` - Implement TypeScript code generation

**Python Target** (1):
- `src/bid-codegen-from-copilot/codegen/targets/python.rs:13` - Implement Python code generation

**WASM Target** (2):
- `src/bid-codegen-from-copilot/codegen/targets/wasm.rs:233` - Implement method
- `src/bid-codegen-from-copilot/codegen/targets/wasm.rs:243` - Return appropriate value

**VM Target** (4):
- `src/bid-codegen-from-copilot/codegen/targets/vm.rs:108` - Integrate with VM's external function registry
- `src/bid-codegen-from-copilot/codegen/targets/vm.rs:136` - Implement actual method logic
- `src/bid-codegen-from-copilot/codegen/targets/vm.rs:144` - Return proper value
- `src/bid-codegen-from-copilot/codegen/targets/vm.rs:191` - Call actual implementation

**Priority**: P3 (Low) - bid-codegen は現在使われていない実験的コード

### 2. Phase 15 関連 - 8 items

**Status**: 🚀 **High Priority** - Active development

**Handle Registry** (2):
- `src/backend/abi_util.rs:120` - Implement handle registry for Phase 15 VM/LLVM backends
- `src/backend/abi_util.rs:128` - Implement handle registry for Phase 15 VM/LLVM backends

**インタープリタ統合** (6):
- `src/runtime/unified_registry.rs:34` - User-defined Box factory will be registered by interpreter
- `src/method_box.rs:239` - グローバル（インタープリタとの統合が必要）
- `src/instance_v2.rs:160` - インタープリター統合時に実装
- `src/instance_v2.rs:274` - SharedNyashBox -> NyashValueの適切な変換を実装
- `src/instance_v2.rs:319` - 適切な変換実装
- `src/instance_v2.rs:375` - 正しいshare_boxセマンティクス実装

**Priority**: P0 (Highest) - Phase 15セルフホスティングの中核

### 3. Plugin System - 7 items

**Status**: 🔧 **Medium Priority** - Maintenance mode

**Plugin Box Legacy** (2):
- `src/runtime/plugin_box_legacy.rs:78` - FFI経由でプラグインにcloneを依頼
- `src/runtime/plugin_box_legacy.rs:100` - プラグインから実際の型名を取得

**Plugin Loader V2** (1):
- `src/runtime/plugin_loader_v2/enabled/extern_functions.rs:121` - Implement full task spawning logic

**Generic Plugin Box** (1):
- `src/bid/generic_plugin_box.rs:125` - Implement proper cloning through v2 plugin loader

**BID Registry** (2):
- `src/bid/registry.rs:70` - Update LoadedPlugin to work with invoke-only plugins
- `src/bid/registry.rs:97` - Create simplified LoadedPlugin without init/abi

**Tests** (1):
- `src/runtime/tests.rs:68` - PluginBox型が削除されたためこのテストはコメントアウト

**Priority**: P1 (High) - プラグインシステムは現在動作中

### 4. WASM Backend - 6 items

**Status**: ⚠️ **Broken** - Needs fixing

**Executor** (2):
- `src/backend/wasm/mod.rs:12` - Fix WASM executor build errors
- `src/backend/wasm/mod.rs:18` - Fix WASM executor build errors

**Codegen** (2):
- `src/backend/wasm/codegen.rs:290` - Add proper field offset calculation
- `src/backend/wasm/codegen.rs:308` - Add proper field offset calculation

**Runtime** (1):
- `src/backend/wasm/runtime.rs:317` - Implement import

**HTTP Server** (1):
- `src/boxes/http_server_box.rs:350` - Actual handler invocation would need method calling infrastructure

**Priority**: P2 (Medium) - WASM機能は現在無効化されている

### 5. Box Factory - 3 items

**Status**: 🔧 **Medium Priority**

- `src/box_factory/plugin.rs:47` - Get list from BoxFactoryRegistry
- `src/box_factory/plugin.rs:55` - Add method to check if registry has any providers
- `src/box_factory/builtin_impls/bool_box.rs:18` - Create nyash-bool-plugin

**Priority**: P1 (High) - Box Factory は Phase 15の一部

### 6. Method Resolution - 3 items

**Status**: 🔧 **Medium Priority**

- `src/mir/builder/calls/method_resolution.rs:109` - Replace with proper method registry lookup
- `src/mir/passes/method_id_inject.rs:10` - Phi/complex flows (dataflow limitation)
- `src/parser/declarations/dependency_helpers.rs:152` - 親クラスのメソッドシグネチャとの比較

**Priority**: P1 (High) - メソッド解決は重要機能

### 7. MIR Passes - 2 items

**Status**: 🔧 **Low Priority**

- `src/mir/passes/cse.rs:4` - SSA update is TODO (counts eliminations without rewriting uses)
- `src/mir/instruction/display.rs:32` - Use callee for type-safe resolution display

**Priority**: P2 (Medium) - 最適化パス関連

### 8. Other - 6 items

**Status**: 🔧 **Mixed Priority**

**AOT** (1):
- `src/backend/aot/mod.rs:85` - Implement full standalone executable generation
- **Priority**: P2 (Medium)

**Box Operators** (1):
- `src/box_operators.rs:14` - Phase 2-4: Static/Dynamic implementations and resolver
- **Priority**: P1 (High)

**Box Trait** (1):
- `src/box_trait.rs:156` - 次のステップで完全実装
- **Priority**: P1 (High)

**JSON** (2):
- `src/boxes/json/mod.rs:219` - FloatBoxが実装されたら有効化
- `src/boxes/json/mod.rs:256` - FloatBoxが実装されたら有効化
- **Priority**: P2 (Medium) - FloatBox は将来機能

**Examples** (1):
- `examples/simple_notepad.rs:50` - テキストエリア全選択
- **Priority**: P3 (Low) - Examples code

**Instance V2** (1):
- `src/instance_v2.rs:353` - type_nameの戻り値型をStringに変更することを検討
- **Priority**: P1 (High)

## Priority Summary

### P0 (Highest) - Phase 15 Core: 8 items
- Handle registry (2)
- インタープリタ統合 (6)

### P1 (High) - Active Development: 17 items
- Plugin System (7)
- Box Factory (3)
- Method Resolution (3)
- Box Operators (1)
- Box Trait (1)
- Instance V2 (1)

### P2 (Medium) - Secondary Features: 12 items
- WASM Backend (6)
- MIR Passes (2)
- AOT (1)
- JSON/FloatBox (2)

### P3 (Low) - Future/Experimental: 11 items
- Code Generation (10)
- Examples (1)

## Recommended Actions

### Immediate (This Sprint)

1. ✅ **Document TODOs** (Done - this file)
2. 🔧 **Phase 15 TODOs** - Focus on P0 items
   - Implement handle registry
   - Complete インタープリタ統合

### Short-term (Next Sprint)

3. 🔧 **Plugin System** - Clean up P1 items
   - Remove commented test or fix it
   - Implement proper cloning

4. 🔧 **Box Factory** - Complete P1 items
   - Implement BoxFactoryRegistry methods

### Medium-term (Next Month)

5. ⚠️ **WASM Backend** - Fix or remove
   - Fix build errors or officially disable
   - Document decision

6. 🔧 **Method Resolution** - Improve P1 items
   - Implement proper method registry

### Long-term (Next Quarter)

7. 🗑️ **Code Generation** - Decide fate
   - Either implement or remove bid-codegen-from-copilot
   - Currently unused experimental code

## Lessons & Best Practices

**From this analysis**:

1. **Don't delete TODO comments hastily**
   - They mark future work
   - They prevent regression
   - They guide implementation

2. **Categorize by priority**
   - P0: Blocking current work
   - P1: Important features
   - P2: Nice to have
   - P3: Future/Experimental

3. **Document intent**
   - Why is this TODO here?
   - What needs to be done?
   - Who can work on it?

## Related Documents

- `dead-code-analysis.md` - Dead code investigation
- `CURRENT_TASK.md` - Current development focus
- Phase 15 docs - Active development plan

---

Generated by: Claude Code (Anthropic)
Date: 2025-10-04
Next review: 2025-11-04 (Monthly)
