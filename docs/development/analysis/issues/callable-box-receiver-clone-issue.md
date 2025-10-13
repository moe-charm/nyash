# CallableBox Receiver Clone Issue

## Summary
CallableBox.call() creates a **clone** of the receiver before invoking the method, which means mutating methods (like `push`, `set`) don't affect the original object.

## Impact
- **Severity**: MEDIUM
- **Affects**: All mutating method calls via CallableBox
- **Workaround**: Use non-mutating methods (get, size, etc.) or avoid CallableBox for mutations

## Root Cause
**File**: `/home/tomoaki/git/hakorune-selfhost/src/runtime/method_router_box/mod.rs`
**Line**: 152-154

```rust
if let Some(recv) = &cb.receiver {
    let recv_vm = VMValue::BoxRef(std::sync::Arc::from(recv.clone_box()));
    //                                                        ^^^^^^^^^^
    //                                                        BUG: Creates a new clone!
    crate::runtime::method_router_box::route(_interp, &recv_vm, &cb.method, &argv)
}
```

## Expected Behavior
```hakorune
local a = new ArrayBox()
local cb = a.methodRef("push", 1)
local args = new ArrayBox()
args.push(42)
cb.call(args)
print(a.size())  // Expected: 1, Actual: 0
```

## Actual Behavior
The push() is called on a **clone** of the array, so the original array remains empty.

## Workaround Example
Use non-mutating methods:
```hakorune
local a = new ArrayBox()
a.push(100)
local cb = a.methodRef("get", 1)
local args = new ArrayBox()
args.push(0)
local result = cb.call(args)  // Works correctly: result = 100
```

## Why This Happens
The `clone_box()` call creates a new instance with a new `BoxBase` (new ID). This is necessary for value semantics but breaks reference semantics for mutations.

## Potential Fixes

### Option 1: Share Reference (Recommended)
Replace `recv.clone_box()` with `recv.share_box()` or direct Arc usage:

```rust
if let Some(recv) = &cb.receiver {
    // Don't clone - use the original reference
    let recv_vm = VMValue::BoxRef(recv.clone());  // Arc clone, not box clone
    crate::runtime::method_router_box::route(_interp, &recv_vm, &cb.method, &argv)
}
```

### Option 2: Store Arc in CallableBox
Change CallableBox to store `Arc<dyn NyashBox>` instead of `Box<dyn NyashBox>`:

```rust
pub struct CallableBox {
    pub(crate) receiver: Option<Arc<dyn NyashBox>>,  // Changed from Box
    // ...
}
```

### Option 3: Document as Limitation
If value semantics are desired for CallableBox, document this as expected behavior and recommend users to work around it.

## Related Files
- `/home/tomoaki/git/hakorune-selfhost/src/boxes/callable/mod.rs` - CallableBox definition
- `/home/tomoaki/git/hakorune-selfhost/src/runtime/method_router_box/mod.rs` - Router implementation
- `/home/tomoaki/git/hakorune-selfhost/apps/tests/test_callable_direct.hako` - Test file (works around issue)

## Test Coverage
Created `/home/tomoaki/git/hakorune-selfhost/apps/tests/test_callable_direct.hako` with 4 tests:
1. ✅ methodRef + arity
2. ✅ call (no args - size)
3. ✅ call (with args - get, non-mutating)
4. ✅ Map.call

All tests PASS with the workaround (using non-mutating methods).

## Priority
- **Current**: LOW (workaround exists)
- **Future**: MEDIUM (if CallableBox becomes widely used for mutations)

## Next Steps
1. Decide on semantic: value vs reference
2. If reference is needed, implement Option 1 or 2
3. Add test for mutating callable (currently would fail)
4. Update documentation to clarify behavior
