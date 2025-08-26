# Plugin Loader Migration Plan

## Overview
Consolidate three plugin loader implementations into a single unified system.

## Current State
1. **plugin_loader_v2.rs** (906 lines) - Main BID-FFI plugin system
2. **plugin_loader.rs** (1217 lines) - Builtin box dynamic loading
3. **plugin_loader_legacy.rs** (299 lines) - Legacy host vtable system

Total: ~2400 lines of code with significant duplication

## Target State
- **plugin_loader_unified.rs** - Single unified loader (~800 lines)
- **plugin_ffi_common.rs** - Shared FFI utilities (~300 lines)
- Total: ~1100 lines (55% reduction)

## Migration Steps

### Phase 1: Infrastructure (DONE)
- [x] Create plugin_loader_unified.rs skeleton
- [x] Create plugin_ffi_common.rs for shared types
- [x] Update runtime/mod.rs
- [x] Basic compilation check

### Phase 2: Common Pattern Extraction
- [ ] Extract TLV encoding/decoding to plugin_ffi_common
- [ ] Extract handle management patterns
- [ ] Extract memory management utilities
- [ ] Extract error handling patterns

### Phase 3: BID Plugin Migration
- [ ] Port PluginBoxV2 implementation
- [ ] Port BID FFI invoke mechanism
- [ ] Port plugin loading logic
- [ ] Port configuration parsing
- [ ] Migrate tests

### Phase 4: Builtin Plugin Migration
- [ ] Port FileBoxProxy and related proxies
- [ ] Port dynamic library loading for builtins
- [ ] Port builtin-specific FFI patterns
- [ ] Migrate feature flags handling

### Phase 5: Legacy Plugin Migration
- [ ] Port host vtable implementation
- [ ] Port legacy box creation
- [ ] Decide on deprecation timeline

### Phase 6: Integration
- [ ] Update all references to old loaders
- [ ] Ensure backward compatibility
- [ ] Performance benchmarking
- [ ] Documentation update

### Phase 7: Cleanup
- [ ] Remove old plugin loaders
- [ ] Remove redundant tests
- [ ] Update CLAUDE.md
- [ ] Final code review

## Technical Decisions

### Unified Plugin Type Detection
```rust
fn detect_plugin_type(path: &str) -> PluginType {
    // 1. Check file extension (.bid, .legacy, .builtin)
    // 2. Check nyash-box.toml for type field
    // 3. Probe for known symbols
    // 4. Default to BID
}
```

### Handle Management Strategy
- Use generic `PluginHandle<T>` for all plugin types
- Centralized handle registry in plugin_ffi_common
- Reference counting with Arc<Mutex<T>>

### Error Handling
- Convert all errors to Result<T, String> at boundaries
- Plugin-specific errors wrapped with context
- Consistent error messages across all plugin types

## Risk Mitigation

### Backward Compatibility
- Keep old loaders under feature flags during migration
- Provide compatibility shims if needed
- Extensive testing with existing plugins

### Performance
- Benchmark before and after migration
- Profile hot paths (box creation, method dispatch)
- Optimize only after correctness verified

### Testing Strategy
1. Unit tests for each component
2. Integration tests with real plugins
3. E2E tests with full applications
4. Regression tests for edge cases

## Success Metrics
- [ ] Code reduction: 50%+ fewer lines
- [ ] All existing tests pass
- [ ] No performance regression
- [ ] Cleaner API surface
- [ ] Better error messages
- [ ] Easier to add new plugin types

## Timeline
- Phase 1-2: Day 1 (Today)
- Phase 3-4: Day 2-3
- Phase 5-6: Day 4
- Phase 7: Day 5

Total: 5 days for complete migration