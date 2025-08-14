# SocketBox State Preservation Fix - Implementation Summary

## Problem Description
In the "Everything is Box" design, stateful boxes like SocketBox were losing their state across field accesses due to `clone_box()` creating new instances instead of sharing references.

**Original Issue**:
```nyash
me.server.bind("127.0.0.1", 8080)  // ✅ SocketBox ID=10, is_server=true
me.server.isServer()                // ❌ SocketBox ID=19, is_server=false (別インスタンス!)
```

## Root Cause Analysis
The issue was in three key locations where `clone_box()` was called:
1. `src/interpreter/core.rs:366` - `resolve_variable()`
2. `src/instance.rs:275` - `get_field()` 
3. `src/interpreter/expressions.rs:779` - `execute_field_access()`

Each access to `me.server` was creating a new SocketBox instance via `clone_box()`, so state changes made to one instance weren't visible in subsequent accesses.

## Solution Implemented

### Phase 1: Type Infrastructure
- Added `SharedNyashBox = Arc<dyn NyashBox>` type alias
- Added `clone_arc()` method to NyashBox trait for Arc conversion

### Phase 2: Data Structure Updates  
- Updated `InstanceBox.fields` from `HashMap<String, Box<dyn NyashBox>>` to `HashMap<String, SharedNyashBox>`
- Updated `NyashInterpreter.local_vars` and `outbox_vars` to use `SharedNyashBox`
- Modified save/restore methods to convert between Arc and Box appropriately

### Phase 3: Core Reference Sharing
- **`resolve_variable()`**: Now returns `SharedNyashBox` and uses `Arc::clone()` instead of `clone_box()`
- **`get_field()`**: Now returns `SharedNyashBox` and uses `Arc::clone()` instead of `clone_box()`  
- **`execute_field_access()`**: Now returns `SharedNyashBox` to preserve sharing

### Phase 4: Target Fix - SocketBox Clone Implementation
**Key Innovation**: Modified SocketBox Clone to share mutable state:

```rust
// Before (problematic):
is_server: Arc::new(Mutex::new(current_is_server)),     // New state container
is_connected: Arc::new(Mutex::new(current_is_connected)), // New state container

// After (fixed):  
is_server: Arc::clone(&self.is_server),    // Share the same state container
is_connected: Arc::clone(&self.is_connected), // Share the same state container
```

This ensures that even when SocketBox instances are cloned, they share the same underlying state containers.

### Phase 5: Interface Compatibility
- Fixed all callers of `resolve_variable()` to handle `SharedNyashBox` return type
- Updated method calls and field access to properly dereference Arc references
- Maintained Box-based external interfaces while using Arc internally

## How the Fix Works

1. **Storage**: Variables and fields are stored as `Arc<dyn NyashBox>` internally
2. **Access**: Field access returns `Arc::clone()` of the stored reference  
3. **Cloning**: When Arc is converted to Box via `clone_box()`, SocketBox creates a new instance BUT shares the same state containers
4. **State Sharing**: All clones of the same original SocketBox share `is_server` and `is_connected` state

**Result**:
```nyash
me.server.bind("127.0.0.1", 8080)  // ✅ SocketBox clone A, is_server=true (shared state)
me.server.isServer()                // ✅ SocketBox clone B, is_server=true (same shared state!)
```

## Testing

### Test Files Created:
- `test_final_validation.nyash` - Replicates the exact issue scenario
- `test_complete_socketbox_fix.nyash` - Comprehensive SocketBox testing
- `test_multiple_stateful_boxes.nyash` - Tests multiple stateful box types
- `test_arc_fix.nyash` - Original issue test case

### Expected Results:
All tests should show that `isServer()` returns `true` after `bind()` is called, confirming that state is preserved across field accesses.

## Impact on Other Stateful Boxes

The fix benefits all stateful boxes:
- **SocketBox**: Fixed with custom Clone implementation
- **HTTPServerBox**: Already uses `Arc::clone()` correctly  
- **FutureBox**: Already uses `Arc::clone()` correctly
- **DebugBox**: Uses derived Clone with Arc fields (works correctly)
- **FileBox**: Uses derived Clone with Arc fields (works correctly)
- **P2PBox**: Designed as `Arc<Mutex<...>>` wrapper (already works correctly)

## Compatibility

- **External Interfaces**: Preserved Box-based APIs for backward compatibility
- **Internal Storage**: Enhanced with Arc-based sharing for stateful objects
- **Performance**: Minimal overhead from Arc reference counting
- **Memory Safety**: Maintains Rust's ownership guarantees

## Conclusion

This fix solves the fundamental issue in the "Everything is Box" design where stateful boxes were losing state due to unnecessary cloning. The hybrid approach maintains interface compatibility while enabling true reference sharing for stateful objects.