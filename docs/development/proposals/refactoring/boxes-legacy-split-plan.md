# handlers/boxes/legacy.rs Split Plan (Phase 15.10-B)

## Analysis

**Current Structure:** Single 518-line file with 3 main functions

**File:** `src/backend/mir_interpreter/handlers/boxes/legacy.rs`

## Function Breakdown

### 1. handle_plugin_invoke (lines 9-128, ~120 lines)
- **Responsibility**: Plugin box method invocation
- **Key Logic**:
  - Unborn guard for plugin instances
  - Plugin loader interaction (PluginBoxV2)
  - Auto-birth no-op handling
  - Fallback to builtin handlers (array/string/map)
  - toString fallback

### 2. handle_box_call (lines 130-359, ~230 lines)
- **Responsibility**: Main BoxCall dispatcher
- **Key Logic**:
  - Unborn guard check
  - Birth lifecycle contracts
  - Plugin routing (NYASH_VM_BOXCALL_PLUGIN_FIRST)
  - Void guard defaults
  - Builtin handler delegation (fields/instance/string/array/map)
  - User instance BoxCall policy gate
  - Dynamic resolution fallback
  - Final plugin box invocation

### 3. invoke_plugin_box (lines 362-517, ~156 lines)
- **Responsibility**: Plugin box bridge implementation
- **Key Logic**:
  - Plugin loader method invocation
  - Birth contracts management
  - Special ConsoleBox.readLine handling
  - Instance fallback handlers (current/is_eof/toString)
  - Dynamic InstanceBox dispatch
  - VoidBox graceful handling

## Split Strategy

Split into **3 files** under `handlers/boxes/legacy/` subdirectory:

```
handlers/boxes/
├── legacy.rs (OLD - 518 lines, to be deleted)
└── legacy/
    ├── mod.rs (~50 lines)
    │   - Main entry point: handle_box_call
    │   - Module re-exports
    ├── plugin_invoke.rs (~125 lines)
    │   - handle_plugin_invoke
    │   - Plugin box invocation entry
    └── plugin_bridge.rs (~165 lines)
        - invoke_plugin_box
        - Fallback helpers (instance_current_fallback, to_string_fallback, etc.)
```

## Implementation Order

1. ✅ Create boxes-legacy-split-plan.md (this file)
2. Create `handlers/boxes/legacy/` directory
3. Extract `plugin_invoke.rs` (lines 9-128)
4. Extract `plugin_bridge.rs` (lines 362-517 + helper methods)
5. Extract `mod.rs` (lines 130-359 + re-exports)
6. Delete old `boxes/legacy.rs`
7. Test boxes/legacy split
8. Commit boxes/legacy refactoring

## Testing Strategy

- **Compilation**: `cargo check`
- **Smoke Test**: `bash tools/smokes/v2/run.sh --profile quick --filter json_lint_vm`
- **Verify**: BoxCall, PluginInvoke, birth() lifecycle all work

## Notes

- All extracted files use `use super::super::super::*;` for imports
- Helper methods (if any) stay with their primary callers
- Module preserves exact API via `pub(crate) use` re-exports
- No changes needed to parent module (boxes/mod.rs)
