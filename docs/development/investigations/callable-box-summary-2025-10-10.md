# CallableBox Fundamental Questions - Quick Summary

## User's Questions & Answers

### Q1: Can I just store CallableBox in ArrayBox.set()? (Why need methodRef?)

**Answer: YES!**

```hakorune
local cb = arr.methodRef("size", 0)  // CREATE CallableBox
local storage = new ArrayBox()
storage.push(cb)                      // STORE in ArrayBox ✅

local retrieved = storage.get(0)      // RETRIEVE
local result = retrieved.call([])     // CALL ✅
```

**Why need methodRef?**
- methodRef **creates** the CallableBox
- ArrayBox.set() **stores** it (like any Box)
- They serve different purposes!

---

### Q2: What do `method` and `arity` fields mean?

**Answer: Runtime Dispatch Metadata**

```rust
pub struct CallableBox {
    pub receiver: Option<Box<dyn NyashBox>>,
    pub method: String,   // 👈 METHOD NAME (e.g., "size", "get", "push")
    pub arity: usize,     // 👈 ARGUMENT COUNT (e.g., 0, 1, 2)
}
```

**Example:**
```hakorune
local cb_size = arr.methodRef("size", 0)
cb_size.arity()  // Returns 0

local cb_get = arr.methodRef("get", 1)
cb_get.arity()   // Returns 1
```

**How they're used:**
1. **method**: Router uses this to dispatch to correct method
2. **arity**: Used for validation (future enhancement)

---

### Q3: Is it difficult to create CallableBox without methodRef?

**Answer: methodRef IS THE STANDARD WAY**

| Approach | Code | Recommended? |
|----------|------|-------------|
| ✅ **WITH methodRef** | `arr.methodRef("size", 0)` | YES - Safe & Easy |
| ❌ **WITHOUT methodRef** | Manual `CallableBox::new(...)` | NO - Complex & Unsafe |

**Why methodRef exists:**
1. Captures RECEIVER (the object)
2. Captures METHOD NAME (which method)
3. Captures ARITY (argument count)
4. Provides SAFETY + CONVENIENCE

**Pattern:**
```
methodRef = Factory Method Pattern
→ Hides complexity
→ Ensures correctness
→ Single entry point
```

---

## Complete Working Test

**File**: `/home/tomoaki/git/hakorune-selfhost/apps/tests/test_callable_fundamental_questions.hako`

**Run**:
```bash
NYASH_DISABLE_PLUGINS=1 ./target/release/hako apps/tests/test_callable_fundamental_questions.hako
```

**Result**: ✅ All 3 questions answered with working code

---

## Key Architecture Points

### CallableBox is a First-Class Box

```rust
impl NyashBox for CallableBox {
    fn clone_box(&self) -> Box<dyn NyashBox> { /* ... */ }
    fn type_name(&self) -> &'static str { "CallableBox" }
    // Can be stored in any Box collection!
}
```

### Creation Flow

```
arr.methodRef("size", 0)
  ↓
MIR: BoxCall(arr, "methodRef", ["size", 0])
  ↓
VM Router: ArrayBox slot 113
  ↓
CallableBox::new(Some(arr), "size", 0)
  ↓
Returns CallableBox
```

### Invocation Flow

```
cb.call([args])
  ↓
MIR: BoxCall(cb, "call", [argsArray])
  ↓
VM Router: CallableBox slot 501
  ↓
Extract cb.receiver and cb.method
  ↓
route(cb.receiver, cb.method, args)
  ↓
Actual method execution
```

---

## Common Use Cases

### 1. Registry Pattern
```hakorune
local commands = new MapBox()
commands.set("size", arr.methodRef("size", 0))
commands.set("get", arr.methodRef("get", 1))

local cb = commands.get("size")
local result = cb.call([])
```

### 2. Event Handlers
```hakorune
local handlers = new ArrayBox()
handlers.push(obj.methodRef("onClick", 1))
handlers.push(obj.methodRef("onHover", 1))

// Trigger all
loop(i < handlers.size()) {
    handlers.get(i).call([event])
}
```

### 3. Map.call Shorthand
```hakorune
registry.set("validator", obj.methodRef("validate", 1))
registry.call("validator", [data])  // Shorthand!
```

---

## Known Issue: Receiver Clone

**Problem**: CallableBox.call() clones the receiver

**Impact**:
```hakorune
local arr = new ArrayBox()
arr.push(10)

local cb = arr.methodRef("push", 1)
cb.call([20])  // Pushes to CLONE, not original!

print(arr.size())  // Still 1 (not 2)
```

**Workaround**: Use non-mutating methods or avoid CallableBox for mutations

**Severity**: MEDIUM (documented, workaround available)

---

## Full Investigation

See: `/home/tomoaki/git/hakorune-selfhost/docs/development/investigations/callable-box-fundamental-investigation-2025-10-10.md` (599 lines)

**Contents**:
- Test results with full code
- Architecture deep dive
- Comparison with other languages (JS/Python/Java)
- Use cases and patterns
- Future enhancements

---

**Date**: 2025-10-10
**Status**: ✅ COMPLETE
**Test Status**: ✅ PASS (all working code examples)
