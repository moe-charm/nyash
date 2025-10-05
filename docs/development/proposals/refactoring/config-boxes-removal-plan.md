# Config Boxes Removal Plan (Phase 15.10-C)

## Analysis

**Target Files:** 4 unused Config box implementations (470 lines total)

## Findings

All 4 Config boxes have **ZERO usage** across the codebase:

### 1. DebugConfigBox (165 lines)
- **Purpose**: JIT debug flags (jit_events, jit_stats, jit_dump, etc.)
- **Status**: OBSOLETE - JIT archived in Phase 15
- **Usage**: None found
- **Decision**: DELETE

### 2. GcConfigBox (95 lines)
- **Purpose**: GC configuration (counting, trace, barrier_strict)
- **Status**: Unused - no references outside definition
- **Usage**: None found
- **Decision**: DELETE

### 3. AotConfigBox (114 lines)
- **Purpose**: AOT compilation config
- **Status**: Unused - no references outside definition
- **Usage**: None found
- **Decision**: DELETE

### 4. AotCompilerBox (89 lines)
- **Purpose**: AOT compiler interface
- **Status**: Unused - no references outside definition
- **Usage**: None found
- **Decision**: DELETE

## Removal Strategy

1. ✅ Verify zero usage (grep confirmed)
2. Delete 4 files:
   - `src/boxes/debug_config_box.rs`
   - `src/boxes/gc_config_box.rs`
   - `src/boxes/aot_config_box.rs`
   - `src/boxes/aot_compiler_box.rs`
3. Remove from `src/boxes/mod.rs`:
   - Module declarations (lines 71, 79, 81)
   - Re-exports (lines 111, 112)
4. Remove from `src/box_trait.rs`:
   - "JitConfigBox" from builtin box list (line 59)
5. Test compilation + smoke test
6. Commit removal

## Expected Impact

- **Lines removed**: ~470
- **Build time**: Slightly faster (4 fewer files to compile)
- **Maintenance**: Reduced complexity
- **Risk**: ZERO (no usage found)

## Testing Strategy

- **Compilation**: `cargo check`
- **Smoke Test**: `bash tools/smokes/v2/run.sh --profile quick --filter json_lint_vm`
- **Verify**: No references to deleted boxes in error messages
