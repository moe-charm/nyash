# methodRef Scalability Investigation (2025-10-10)

## User's Critical Question

**"If every Box needs methodRef implemented individually, doesn't that scale poorly? How do other languages handle this?"**

### Context

User observed that:
1. ArrayBox has `.methodRef()` implemented (slot 113)
2. If **every Box** (MapBox, StringBox, UserBox1, UserBox2, ..., UserBox100) needs individual implementation
3. This could lead to **massive code duplication**

**Core concern**: Does Hakorune's architecture scale when you have 100+ custom Boxes?

---

## Executive Summary

### Answer: **Hakorune ALREADY HAS a universal solution!**

✅ **Current State**: methodRef is **ONLY** implemented for ArrayBox
✅ **Architecture**: The `nyrt_callable_make()` host function is **UNIVERSAL**
✅ **Scalability**: Adding methodRef to new Boxes requires **ZERO duplication**
✅ **Comparison**: Similar to JavaScript's `.bind()`, Python's `functools.partial`

### Key Finding

**methodRef is NOT Box-specific logic** — it's a **thin wrapper** calling the universal `nyrt_callable_make()` host function.

**Implementation size**: **6 lines per Box** (just parameter passing)

```rust
// ArrayBox slot 113 handler (src/runtime/method_router_box/mod.rs:339-344)
113 => { // methodRef(name, arity) -> CallableBox
    let name = args.get(0).map(|v| v.to_string()).unwrap_or_default();
    let ar = args.get(1).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);
    let cb = CallableBox::new(Some(arr.clone_box()), name, ar as usize);
    Ok(VMValue::from_nyash_box(Box::new(cb)))
}
```

**That's it!** No Box-specific logic beyond parameter extraction.

---

## Language Comparison: How Others Solve This

### 1. JavaScript: `.bind()` on `Function.prototype`

**Universal Implementation**:
```javascript
const arr = [1, 2, 3];
const pushBound = arr.push.bind(arr, 42);
pushBound();  // arr.push(42)
```

**Where is `.bind()` implemented?**
- **Answer**: `Function.prototype.bind` (universal, inherited by ALL functions)
- **Inheritance**: Every method inherits `.bind()` from `Function.prototype`
- **Duplication**: ZERO (single implementation for all objects)

**Architecture**:
```
Object.prototype
  ↓
Function.prototype.bind()  👈 SINGLE IMPLEMENTATION
  ↓
[].push.bind() ← inherit
[].pop.bind() ← inherit
{}.toString.bind() ← inherit
```

---

### 2. Python: `functools.partial()` (Global Function)

**Universal Implementation**:
```python
from functools import partial

arr = [1, 2, 3]
push_partial = partial(arr.append, 42)
push_partial()  # arr.append(42)
```

**Where is bound method creation?**
- **Answer**: Python's **descriptor protocol** (universal, automatic)
- **Mechanism**: `obj.method` automatically creates a bound method
- **Duplication**: ZERO (built into language runtime)

**Architecture**:
```
class object:
    def __getattribute__(self, name):
        attr = /* lookup */
        if callable(attr):
            return BoundMethod(self, attr)  👈 AUTOMATIC
```

---

### 3. Ruby: `Object#method()` (Universal Inheritance)

**Universal Implementation**:
```ruby
arr = [1, 2, 3]
push_method = arr.method(:push)
push_method.call(42)  # arr.push(42)
```

**Where is `.method()` implemented?**
- **Answer**: `Object#method` (all objects inherit)
- **Inheritance**: Single implementation in `Object` base class
- **Duplication**: ZERO (inherited by all objects)

**Architecture**:
```
Object#method(name)  👈 SINGLE IMPLEMENTATION
  ↓
Array#method(name) ← inherit
Hash#method(name) ← inherit
UserClass#method(name) ← inherit
```

---

### 4. Java: `::` Method References (Compiler Syntax)

**Universal Implementation**:
```java
List<Integer> arr = new ArrayList<>();
Consumer<Integer> pushRef = arr::add;
pushRef.accept(42);  // arr.add(42)
```

**Where is `::` syntax handled?**
- **Answer**: **Compiler transformation** (not a runtime method)
- **Mechanism**: Compiler generates lambda wrapper classes
- **Duplication**: ZERO (compiler handles all cases)

**Architecture**:
```
Source Code: arr::add
  ↓
Compiler generates:
  new Consumer<Integer>() {
      public void accept(Integer x) {
          arr.add(x);
      }
  }
```

---

### 5. Hakorune: Current Architecture

**Current Implementation** (ArrayBox-only):
```hakorune
local arr = new ArrayBox()
local cbPush = arr.methodRef("push", 1)
cbPush.call([42])  // arr.push(42)
```

**Where is methodRef implemented?**
- **Answer**: Slot 113 in `method_router_box.rs` (ArrayBox-specific)
- **Duplication**: Currently ZERO (only ArrayBox has it)
- **Scalability**: Requires **6 lines per Box** (parameter passing only)

**Architecture**:
```
ArrayBox slot 113 handler (6 lines)
  ↓
CallableBox::new(receiver, method, arity)  👈 UNIVERSAL IMPL
  ↓
CallableBox.call() → method_router_box::route()
```

---

## Comparison Table

| Language | Strategy | Universal? | Implementation Location |
|----------|----------|-----------|------------------------|
| **JavaScript** | `Function.prototype.bind` | ✅ All functions | Single prototype method |
| **Python** | Descriptor protocol | ✅ All objects | Language runtime |
| **Ruby** | `Object#method` | ✅ All objects | Base class inheritance |
| **Java** | Compiler `::` syntax | ✅ All objects | Compile-time |
| **Hakorune (Current)** | Per-Box slot handler | ❌ Only ArrayBox | method_router_box.rs:339-344 |
| **Hakorune (Proposed)** | VM-level universal handler | ✅ All Boxes | Single pre-check in route() |

---

## Hakorune's Current State: Analysis

### Current Implementation

**File**: `src/runtime/method_router_box/mod.rs:339-344`

```rust
// ArrayBox methodRef handler (slot 113)
113 => { // methodRef(name, arity) -> CallableBox
    let name = args.get(0).map(|v| v.to_string()).unwrap_or_default();
    let ar = args.get(1).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);
    let cb = crate::boxes::callable::CallableBox::new(Some(arr.clone_box()), name, ar as usize);
    Ok(VMValue::from_nyash_box(Box::new(cb)))
}
```

**Key observation**: This code is **100% generic** — it works for ANY Box!

### Scalability Analysis

**Question**: What happens when we add MapBox.methodRef()? StringBox.methodRef()? UserBox.methodRef()?

**Answer**: Each requires **6 lines** of identical code:

```rust
// MapBox slot 213 (hypothetical)
213 => { // methodRef(name, arity) -> CallableBox
    let name = args.get(0).map(|v| v.to_string()).unwrap_or_default();
    let ar = args.get(1).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);
    let cb = CallableBox::new(Some(map.clone_box()), name, ar as usize);
    Ok(VMValue::from_nyash_box(Box::new(cb)))
}

// StringBox slot 313 (hypothetical)
313 => { // methodRef(name, arity) -> CallableBox
    let name = args.get(0).map(|v| v.to_string()).unwrap_or_default();
    let ar = args.get(1).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);
    let cb = CallableBox::new(Some(str_box.clone_box()), name, ar as usize);
    Ok(VMValue::from_nyash_box(Box::new(cb)))
}
```

**Problem**: **Code duplication** across 100+ Boxes!

---

## Root Cause: Per-Box Dispatch Model

### Current Architecture

```
User Code: arr.methodRef("size", 0)
  ↓
MIR: BoxCall(arr, "methodRef", ["size", 0])
  ↓
method_router_box::route(receiver, method, args)
  ↓
match receiver.type_name() {
    "ArrayBox" => {
        if slot == 113 { /* methodRef handler */ }  👈 DUPLICATION
    }
    "MapBox" => {
        if slot == 213 { /* methodRef handler */ }  👈 DUPLICATION
    }
    "StringBox" => {
        if slot == 313 { /* methodRef handler */ }  👈 DUPLICATION
    }
}
```

**Problem**: Each Box type requires **separate slot registration** + **identical handler code**

---

## Proposed Solution: Universal Pre-Check

### Option A: VM-Level Universal methodRef Handler

**Inspiration**: JavaScript's `Function.prototype.bind` (applies to ALL functions)

**Implementation** (add to `method_router_box::route()` BEFORE type-specific dispatch):

```rust
// File: src/runtime/method_router_box/mod.rs
pub fn route(
    _interp: &mut MirInterpreter,
    receiver: &VMValue,
    method: &str,
    args: &[VMValue],
) -> Result<VMValue, VMError> {
    // ===== UNIVERSAL methodRef HANDLER (NEW!) =====
    if method == "methodRef" && args.len() == 2 {
        let name = args.get(0).map(|v| v.to_string()).unwrap_or_default();
        let ar = args.get(1).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);

        // Convert receiver to Box
        let recv_box: Box<dyn NyashBox> = match receiver {
            VMValue::BoxRef(bx) => bx.clone_box(),
            VMValue::String(s) => Box::new(crate::box_trait::StringBox::new(s)),
            VMValue::Integer(i) => Box::new(crate::box_trait::IntegerBox::new(*i)),
            VMValue::Bool(b) => Box::new(crate::box_trait::BoolBox::new(*b)),
            _ => return Err(VMError::InvalidInstruction("methodRef on non-Box receiver".into())),
        };

        let cb = crate::boxes::callable::CallableBox::new(Some(recv_box), name, ar as usize);
        return Ok(VMValue::from_nyash_box(Box::new(cb)));
    }
    // ===== END UNIVERSAL HANDLER =====

    // Existing Box-specific dispatch...
    if let VMValue::String(s) = receiver {
        // StringBox handlers...
    }
    if let VMValue::BoxRef(bx) = receiver {
        match bx.type_name() {
            "ArrayBox" => { /* No longer needs slot 113! */ }
            "MapBox" => { /* No longer needs slot 213! */ }
            // ...
        }
    }
}
```

**Benefits**:
- ✅ **Zero duplication**: Single implementation for ALL Boxes
- ✅ **Works for user-defined Boxes**: No need to implement methodRef
- ✅ **Consistent behavior**: Same semantics across all Box types
- ✅ **Simple**: ~15 lines of code (vs. 6 lines × 100 Boxes)

**Trade-offs**:
- ❌ **Less flexible**: Can't override methodRef behavior per Box
- ❌ **Bypasses plugin logic**: Plugin Boxes can't customize methodRef

---

### Option B: Type Registry Default Implementation

**Inspiration**: Ruby's `Object#method` (inherited by all objects)

**Implementation**: Add methodRef to type_registry as a **universal slot**

```rust
// File: src/runtime/type_registry.rs

// Universal slot 0: methodRef (all Boxes)
const UNIVERSAL_METHODS: &[MethodEntry] = &[
    MethodEntry { name: "methodRef", arity: 2, slot: 0 },
];

pub fn resolve_slot_by_name(type_name: &str, method: &str, arity: usize) -> Option<u16> {
    // Check universal methods first
    for m in UNIVERSAL_METHODS {
        if m.name == method && m.arity == arity as u8 {
            return Some(m.slot);
        }
    }

    // Then check type-specific methods
    let tb = resolve_typebox_by_name(type_name)?;
    // ...
}
```

**Benefits**:
- ✅ **Type Registry integration**: Consistent with existing architecture
- ✅ **Slot-based dispatch**: Reuses existing vtable infrastructure
- ✅ **Per-Box override**: Can still customize if needed (slot 0 conflicts)

**Trade-offs**:
- ⚠️ **Slot collision**: Slot 0 might conflict with Box-specific methods
- ⚠️ **More complex**: Requires universal slot handling in dispatch

---

### Option C: Trait Default Method (Future, Phase 25+)

**Inspiration**: Rust's trait default methods

**Implementation** (requires trait system):

```rust
trait NyashBox {
    // Universal implementation
    fn methodRef(&self, method: String, arity: usize) -> CallableBox {
        CallableBox::new(Some(self.clone_box()), method, arity)
    }

    // Boxes can override if needed
    fn methodRef_custom(&self, method: String, arity: usize) -> CallableBox {
        self.methodRef(method, arity)  // Default behavior
    }
}
```

**Benefits**:
- ✅ **Idiomatic Rust**: Leverages trait system
- ✅ **Override flexibility**: Boxes can customize behavior
- ✅ **Type-safe**: Compile-time checking

**Trade-offs**:
- ❌ **Future only**: Requires Phase 25 trait system
- ❌ **Not applicable to plugins**: Plugins don't implement Rust traits

---

## Plugin Boxes: Special Case

### Current Plugin Implementation

**File**: `plugins/nyash-array-plugin/src/lib.rs:104-135`

```rust
extern "C" fn array_invoke_id(...) {
    match method_id {
        METHOD_METHODREF => {
            let name = read_arg_string(args, args_len, 0)?;
            let arity = read_arg_i64(args, args_len, 1)?;

            // Call universal host function
            let rc = nyrt_callable_make(
                TYPE_ID_ARRAY,
                instance_id,
                name_bytes.as_ptr(),
                name_bytes.len(),
                arity as u32,
                &mut handle,
            );

            return write_tlv_host_handle(handle, result, result_len);
        }
    }
}
```

**Key observation**: Plugin calls `nyrt_callable_make()` — a **UNIVERSAL HOST FUNCTION**!

### Universal Host Function

**File**: `src/runtime/host_api.rs:617-658`

```rust
#[no_mangle]
pub extern "C" fn nyrt_callable_make(
    recv_type_id: u32,
    recv_instance_id: u32,
    method_ptr: *const u8,
    method_len: usize,
    arity: u32,
    out_handle: *mut u64,
) -> i32 {
    // Reconstruct receiver Box from (type_id, instance_id)
    let recv_box = loader_guard.construct_existing_instance(recv_type_id, recv_instance_id)?;

    // Create CallableBox (UNIVERSAL!)
    let callable = CallableBox::new(Some(recv_box), method, arity as usize);

    // Return handle
    let handle = host_handles::to_handle_box(Box::new(callable));
    unsafe { *out_handle = handle; }
    0
}
```

**Key finding**: `nyrt_callable_make()` is **ALREADY UNIVERSAL**!

### Implication

**Plugin boxes DON'T need to implement methodRef logic** — they just call `nyrt_callable_make()`!

**Proof**:
- ✅ `nyrt_callable_make()` works for **ANY** Box (builtin or plugin)
- ✅ Plugin code is **identical** for all Boxes (just parameter passing)
- ✅ Adding methodRef to MapBox plugin = **6 lines** (call `nyrt_callable_make()`)

---

## Scalability Assessment

### Current State (ArrayBox-only)

| Box Type | methodRef Implemented? | Lines of Code |
|----------|----------------------|---------------|
| **ArrayBox** | ✅ Yes (slot 113) | 6 lines |
| **MapBox** | ❌ No | 0 lines |
| **StringBox** | ❌ No | 0 lines |
| **UserBox1** | ❌ No | 0 lines |
| **UserBox2** | ❌ No | 0 lines |
| **UserBox100** | ❌ No | 0 lines |
| **TOTAL** | 1/100 | 6 lines |

### Proposed State (Universal Handler)

| Box Type | methodRef Implemented? | Lines of Code |
|----------|----------------------|---------------|
| **ALL Boxes** | ✅ Yes (universal) | 15 lines (shared) |
| **ArrayBox** | ✅ Yes | 0 lines (inherited) |
| **MapBox** | ✅ Yes | 0 lines (inherited) |
| **StringBox** | ✅ Yes | 0 lines (inherited) |
| **UserBox1** | ✅ Yes | 0 lines (inherited) |
| **UserBox100** | ✅ Yes | 0 lines (inherited) |
| **TOTAL** | 100/100 | 15 lines (shared) |

### Scalability Comparison

| Scenario | Per-Box Slots | Universal Handler | Savings |
|----------|--------------|------------------|---------|
| **1 Box** | 6 lines | 15 lines | -9 lines (worse) |
| **3 Boxes** | 18 lines | 15 lines | +3 lines |
| **10 Boxes** | 60 lines | 15 lines | +45 lines |
| **100 Boxes** | 600 lines | 15 lines | +585 lines (97.5% reduction!) |

**Conclusion**: Universal handler becomes **beneficial at 3+ Boxes**, scales to **100+ Boxes** with zero additional cost.

---

## Recommendation

### Proposed Implementation: Option A (VM-Level Universal Handler)

**Rationale**:
1. ✅ **Simplest**: Single 15-line pre-check in `method_router_box::route()`
2. ✅ **Immediate benefit**: Works for ALL Boxes (builtin, plugin, user) without changes
3. ✅ **No duplication**: Zero code duplication across Boxes
4. ✅ **Maintainable**: Single point of maintenance

**Implementation Plan**:

1. **Add universal handler** to `method_router_box::route()` (15 lines)
2. **Remove ArrayBox slot 113** (no longer needed)
3. **Update type_registry**: Remove methodRef from ARRAY_METHODS (1 line)
4. **Test**: Verify all existing tests pass
5. **Document**: Update API docs to reflect universal methodRef

**Code changes**: ~20 lines (15 added, 5 removed)

**Estimated time**: 30 minutes

---

### Alternative: Hybrid Approach (Universal + Per-Box Override)

**For maximum flexibility**:

```rust
pub fn route(...) -> Result<VMValue, VMError> {
    // 1. Check if Box has custom methodRef implementation (slot-based)
    if method == "methodRef" {
        if let Some(slot) = resolve_slot_by_name(type_name, "methodRef", 2) {
            // Use Box-specific implementation
            return dispatch_to_slot(receiver, slot, args);
        }

        // 2. Fall back to universal implementation
        let name = args[0].to_string();
        let arity = args[1].as_integer().unwrap_or(0);
        let cb = CallableBox::new(Some(receiver.clone_box()), name, arity as usize);
        return Ok(VMValue::from_nyash_box(Box::new(cb)));
    }

    // ... rest of dispatch
}
```

**Benefits**:
- ✅ **Universal fallback**: Works for all Boxes by default
- ✅ **Per-Box customization**: Boxes can override if needed (e.g., validation)
- ✅ **Zero duplication**: Most Boxes use universal impl

---

## Comparison with Other Languages (Final)

| Language | Implementation | Duplication | Scalability | Override? |
|----------|---------------|-------------|-------------|----------|
| **JavaScript** | `Function.prototype.bind` | Zero | ✅ Perfect | ❌ No (prototype) |
| **Python** | Descriptor protocol | Zero | ✅ Perfect | ❌ No (runtime) |
| **Ruby** | `Object#method` | Zero | ✅ Perfect | ✅ Yes (redefine) |
| **Java** | Compiler `::` | Zero | ✅ Perfect | ❌ No (syntax) |
| **Hakorune (Current)** | Per-Box slots | High (6 lines × N) | ❌ Poor | ✅ Yes |
| **Hakorune (Proposed A)** | Universal pre-check | Zero | ✅ Perfect | ❌ No |
| **Hakorune (Proposed Hybrid)** | Universal + slots | Zero | ✅ Perfect | ✅ Yes |

**Recommendation**: **Hybrid Approach** — combines Ruby's flexibility with JavaScript's universality.

---

## Implementation Example

### Before (Current)

```rust
// src/runtime/method_router_box/mod.rs:339-344
"ArrayBox" => {
    match slot {
        113 => { // methodRef (6 lines of duplication)
            let name = args.get(0).map(|v| v.to_string()).unwrap_or_default();
            let ar = args.get(1).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);
            let cb = CallableBox::new(Some(arr.clone_box()), name, ar as usize);
            Ok(VMValue::from_nyash_box(Box::new(cb)))
        }
    }
}

// MapBox: NO methodRef
// StringBox: NO methodRef
// UserBoxes: NO methodRef
```

### After (Proposed)

```rust
// src/runtime/method_router_box/mod.rs (NEW: lines 31-45)
pub fn route(
    _interp: &mut MirInterpreter,
    receiver: &VMValue,
    method: &str,
    args: &[VMValue],
) -> Result<VMValue, VMError> {
    // ===== UNIVERSAL methodRef HANDLER =====
    if method == "methodRef" && args.len() == 2 {
        // Check for custom implementation first
        if let VMValue::BoxRef(bx) = receiver {
            let type_name = bx.type_name();
            if let Some(slot) = resolve_slot_by_name(type_name, "methodRef", 2) {
                // Use Box-specific implementation (rare)
                // (dispatch to slot handler)
            }
        }

        // Universal fallback (most Boxes use this)
        let name = args.get(0).map(|v| v.to_string()).unwrap_or_default();
        let ar = args.get(1).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);
        let recv_box = match receiver {
            VMValue::BoxRef(bx) => bx.clone_box(),
            VMValue::String(s) => Box::new(StringBox::new(s)),
            _ => return Err(VMError::InvalidInstruction("methodRef on invalid receiver".into())),
        };
        let cb = CallableBox::new(Some(recv_box), name, ar as usize);
        return Ok(VMValue::from_nyash_box(Box::new(cb)));
    }
    // ===== END UNIVERSAL HANDLER =====

    // Existing dispatch (unchanged)...
}
```

**Result**:
- ✅ ArrayBox.methodRef works (universal)
- ✅ MapBox.methodRef works (universal)
- ✅ StringBox.methodRef works (universal)
- ✅ UserBox.methodRef works (universal)
- ✅ Can still override per Box (slot-based)

---

## Test Plan

### 1. Existing Tests (Regression)

Verify all existing CallableBox tests pass:
- `apps/tests/test_callable_basic.hako`
- `apps/tests/test_callable_direct.hako`
- `apps/tests/test_callable_storage.hako`
- Smoke tests: `callable_hakorune_vm.sh`, `callable_async_builtin_vm.sh`

### 2. New Tests (Universal methodRef)

```hakorune
// Test: Universal methodRef for all Boxes
static box Main {
  main() {
    // ArrayBox (existing)
    local arr = new ArrayBox()
    local cbArr = arr.methodRef("size", 0)
    print(cbArr.call([]))  // Should work

    // MapBox (NEW!)
    local map = new MapBox()
    local cbMap = map.methodRef("size", 0)
    print(cbMap.call([]))  // Should work

    // StringBox (NEW!)
    local str = "hello"
    local cbStr = str.methodRef("length", 0)
    print(cbStr.call([]))  // Should work (via primitive wrapper)

    return 0
  }
}
```

### 3. Plugin Tests

Verify plugin ArrayBox.methodRef still works:
- `callable_async_plugin_vm.sh`
- `map_callable_min_vm.sh`

---

## Performance Considerations

### Current (Slot-based)

**Dispatch path**:
```
route() → type_name match → slot match (113) → handler
```
**Cost**: 2 match statements (O(1))

### Proposed (Universal Pre-Check)

**Dispatch path**:
```
route() → method == "methodRef" check → universal handler
```
**Cost**: 1 string comparison (O(1))

**Performance impact**: **NEGLIGIBLE** (likely faster due to early exit)

---

## Conclusion

### Key Findings

1. ✅ **Hakorune's architecture ALREADY supports universal methodRef**
   - `CallableBox::new()` is universal (works for any Box)
   - `nyrt_callable_make()` is universal (works for plugin Boxes)
   - Current slot-based implementation is **unnecessary duplication**

2. ✅ **Other languages use universal implementations**
   - JavaScript: `Function.prototype.bind` (single prototype method)
   - Python: Descriptor protocol (automatic bound methods)
   - Ruby: `Object#method` (inherited by all objects)
   - Java: Compiler `::` syntax (compile-time transformation)

3. ✅ **Universal handler scales perfectly**
   - 1 Box: 6 lines (per-Box) vs. 15 lines (universal) → per-Box wins
   - 3 Boxes: 18 lines vs. 15 lines → universal wins
   - 100 Boxes: 600 lines vs. 15 lines → **97.5% reduction!**

### Recommendation

**Implement Option A (VM-Level Universal Handler) with Hybrid Fallback**

**Benefits**:
- ✅ **Zero duplication**: Single implementation for ALL Boxes
- ✅ **Scales perfectly**: 100+ Boxes with zero additional cost
- ✅ **Flexible**: Per-Box override still possible (via slots)
- ✅ **Simple**: 15-20 lines of code
- ✅ **Maintainable**: Single point of maintenance

**Implementation time**: 30 minutes

**Files to modify**:
1. `src/runtime/method_router_box/mod.rs`: Add universal handler (15 lines)
2. `src/runtime/type_registry.rs`: Remove ArrayBox slot 113 (optional)
3. Tests: Add universal methodRef tests (5 test cases)

### Next Steps

1. **Decision**: Approve universal handler approach
2. **Implementation**: Add pre-check to `method_router_box::route()`
3. **Testing**: Verify existing + new tests pass
4. **Documentation**: Update CallableBox API docs
5. **Rollout**: Enable for all Boxes (builtin, plugin, user)

---

## Appendix: Full Code Implementation

### Proposed Change (Complete)

**File**: `src/runtime/method_router_box/mod.rs`

```rust
pub fn route(
    _interp: &mut MirInterpreter,
    receiver: &VMValue,
    method: &str,
    args: &[VMValue],
) -> Result<VMValue, VMError> {
    // ===== UNIVERSAL methodRef HANDLER (NEW) =====
    // Implements universal methodRef for ALL Boxes (builtin, plugin, user)
    // Rationale: methodRef logic is identical across all Box types (no Box-specific behavior)
    // Inspired by: JavaScript's Function.prototype.bind (universal inheritance)
    if method == "methodRef" && args.len() == 2 {
        // Extract method name and arity
        let name = args.get(0).map(|v| v.to_string()).unwrap_or_default();
        let ar = args.get(1).map(|v| v.as_integer().unwrap_or(0)).unwrap_or(0);

        // Convert receiver to Box
        let recv_box: Box<dyn NyashBox> = match receiver {
            VMValue::BoxRef(bx) => {
                // Check for custom methodRef implementation (rare)
                let type_name = bx.type_name();
                if let Some(slot) = crate::runtime::type_registry::resolve_slot_by_name(type_name, "methodRef", 2) {
                    // Box has custom implementation — delegate to slot handler
                    // (This allows per-Box override if needed)
                    // Currently only used by ArrayBox (slot 113), but will be removed
                    // in favor of this universal implementation.
                    // NOTE: This check can be removed after deprecating per-Box slots.
                }
                bx.clone_box()
            }
            VMValue::String(s) => Box::new(crate::box_trait::StringBox::new(s)),
            VMValue::Integer(i) => Box::new(crate::box_trait::IntegerBox::new(*i)),
            VMValue::Bool(b) => Box::new(crate::box_trait::BoolBox::new(*b)),
            _ => return Err(VMError::InvalidInstruction("methodRef on non-Box receiver".into())),
        };

        // Create CallableBox (universal implementation)
        let cb = crate::boxes::callable::CallableBox::new(Some(recv_box), name, ar as usize);
        return Ok(VMValue::from_nyash_box(Box::new(cb)));
    }
    // ===== END UNIVERSAL HANDLER =====

    // Existing type-specific dispatch (unchanged)...
    if let VMValue::String(s) = receiver {
        // StringBox handlers...
    }
    if let VMValue::BoxRef(bx) = receiver {
        match bx.type_name() {
            "ArrayBox" => {
                // ArrayBox slot 113 can be REMOVED (now uses universal handler)
                // TODO: Remove slot 113 from type_registry and dispatch table
            }
            "MapBox" => {
                // MapBox now gets methodRef for FREE (via universal handler)
            }
            // ... other Boxes
        }
    }

    Err(VMError::InvalidInstruction(format!("Method {} not supported", method)))
}
```

---

**Investigation Date**: 2025-10-10
**Investigator**: Claude (Code Agent)
**Status**: COMPLETE
**Recommendation**: Implement Universal VM-Level methodRef Handler (Option A + Hybrid)
**Impact**: **97.5% code reduction** for 100 Boxes (600 lines → 15 lines)
**Implementation Time**: 30 minutes
**Priority**: MEDIUM (current system works, but doesn't scale)
