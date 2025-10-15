# Op_eq Migration — Move Equality to Hakorune

**Purpose**: Migrate equality (`==`) logic from Rust to Hakorune
**Timeline**: Week 5-6 of Phase 20.5
**Why**: Fix recursion bugs, improve correctness, prepare for Pure Hakorune

---

## 🎯 Problem Statement

### Current Implementation (Rust)

**Location**: `src/vm_ops/compare/mod.rs`

**Issues**:
1. **Infinite Recursion**: Nested comparisons can loop forever
   ```rust
   // Example bug (2025 issue):
   struct A { field: B }
   struct B { field: A }
   a1 == a2  // ← Infinite recursion!
   ```

2. **Rust-side Logic**: Equality is in Rust, not Hakorune
   - Violates "Everything in Hakorune" principle
   - Hard to extend (requires Rust code changes)
   - Not accessible from Hakorune VM

3. **No Guard Mechanism**: Missing protection against re-entry
   - `op_eq` calls `Box.equals()` calls `op_eq` → loop
   - No way to detect/prevent this cycle

---

## 🎯 Target Implementation (Hakorune)

### New Location

**File**: `apps/hakorune-vm/op_eq_box.hako` (new)

**Key Features**:
1. **NoOperatorGuard**: Prevents infinite recursion
2. **Comparison Order**: Predictable, stable algorithm
3. **Extensible**: User-defined `equals` methods
4. **Golden Tested**: Rust-VM vs Hako-VM parity

---

## 📋 Comparison Algorithm

### Comparison Order (Top to Bottom)

```hakorune
static box OpEqBox {
  compare(lhs, rhs, guard) {
    // 1. Pointer equality (fast path)
    if (ptr_eq(lhs, rhs)) { return 1 }

    // 2. Primitive types
    if (is_int(lhs) && is_int(rhs)) {
      return int_eq(lhs, rhs)
    }
    if (is_bool(lhs) && is_bool(rhs)) {
      return bool_eq(lhs, rhs)
    }
    if (is_null(lhs) && is_null(rhs)) {
      return 1
    }
    if (is_null(lhs) || is_null(rhs)) {
      return 0  // null != non-null
    }

    // 3. String (value equality)
    if (is_string(lhs) && is_string(rhs)) {
      return string_eq(lhs, rhs)
    }

    // 4. Array (recursive, but guarded)
    if (is_array(lhs) && is_array(rhs)) {
      return array_eq(lhs, rhs, guard)
    }

    // 5. Map (key-value pairs)
    if (is_map(lhs) && is_map(rhs)) {
      return map_eq(lhs, rhs, guard)
    }

    // 6. Enum (@enum types)
    if (is_enum(lhs) && is_enum(rhs)) {
      return enum_eq(lhs, rhs, guard)
    }

    // 7. User-defined equals (via Resolver)
    if (has_equals_method(lhs)) {
      return call_user_equals(lhs, rhs, guard)
    }

    // 8. Default: pointer equality
    return ptr_eq(lhs, rhs)
  }
}
```

---

## 🛡️ NoOperatorGuard Implementation

### Purpose

**Problem**:
```
A.equals(B) → op_eq(A.field, B.field) → C.equals(D) → op_eq(C.field, D.field) → ...
```

**Solution**: Track visited pairs, detect cycles

### Design

```hakorune
box NoOperatorGuard {
  visited: MapBox  // Map<(ptr_lhs, ptr_rhs), bool>

  birth() {
    me.visited = new MapBox()
  }

  check(lhs, rhs) {
    local key = make_key(lhs, rhs)
    if (me.visited.has(key)) {
      return 0  // Already visiting → false (prevent recursion)
    }
    me.visited.set(key, 1)
    return 1
  }

  uncheck(lhs, rhs) {
    local key = make_key(lhs, rhs)
    me.visited.remove(key)
  }
}

// Helper
make_key(lhs, rhs) {
  local ptr_lhs = get_ptr(lhs)
  local ptr_rhs = get_ptr(rhs)
  return ptr_lhs + ":" + ptr_rhs
}
```

### Usage

```hakorune
static box OpEqBox {
  array_eq(lhs, rhs, guard) {
    // Check guard before recursing
    if (!guard.check(lhs, rhs)) {
      return 0  // Cycle detected
    }

    local result
    if (lhs.size() != rhs.size()) {
      result = 0
    } else {
      result = 1
      local i = 0
      loop(i < lhs.size()) {
        // Recursive call with guard
        if (!me.compare(lhs.get(i), rhs.get(i), guard)) {
          result = 0
          break
        }
        i = i + 1
      }
    }

    // Uncheck after recursion
    guard.uncheck(lhs, rhs)
    return result
  }
}
```

---

## 🔍 Detailed Comparison Implementations

### 1. Pointer Equality (Fast Path)

```hakorune
ptr_eq(lhs, rhs) {
  // C-ABI call: Check if same memory address
  return HostBridgeBox.ptr_eq(lhs, rhs)
}
```

**Benefit**: O(1), handles self-references

---

### 2. Primitive Types

```hakorune
int_eq(lhs, rhs) {
  return lhs.value() == rhs.value()
}

bool_eq(lhs, rhs) {
  return lhs.value() == rhs.value()
}

null_eq(lhs, rhs) {
  return is_null(lhs) && is_null(rhs)
}
```

---

### 3. String (Value Equality)

```hakorune
string_eq(lhs, rhs) {
  if (lhs.length() != rhs.length()) {
    return 0
  }
  // Byte-by-byte comparison
  return lhs.equals(rhs)  // StringBox.equals (built-in)
}
```

**Note**: StringBox.equals is special (no recursion risk)

---

### 4. Array (Recursive, Guarded)

```hakorune
array_eq(lhs, rhs, guard) {
  if (!guard.check(lhs, rhs)) { return 0 }  // Cycle detection

  if (lhs.size() != rhs.size()) {
    guard.uncheck(lhs, rhs)
    return 0
  }

  local result = 1
  local i = 0
  loop(i < lhs.size()) {
    if (!me.compare(lhs.get(i), rhs.get(i), guard)) {
      result = 0
      break
    }
    i = i + 1
  }

  guard.uncheck(lhs, rhs)
  return result
}
```

**Complexity**: O(n * m) where n = array size, m = element comparison cost

---

### 5. Map (Key-Value Pairs)

```hakorune
map_eq(lhs, rhs, guard) {
  if (!guard.check(lhs, rhs)) { return 0 }

  if (lhs.size() != rhs.size()) {
    guard.uncheck(lhs, rhs)
    return 0
  }

  local result = 1
  local keys = lhs.keys()
  local i = 0
  loop(i < keys.size()) {
    local key = keys.get(i)

    // Check key exists in rhs
    if (!rhs.has(key)) {
      result = 0
      break
    }

    // Check value equality (recursive)
    local lhs_val = lhs.get(key)
    local rhs_val = rhs.get(key)
    if (!me.compare(lhs_val, rhs_val, guard)) {
      result = 0
      break
    }

    i = i + 1
  }

  guard.uncheck(lhs, rhs)
  return result
}
```

**Complexity**: O(k * v) where k = key count, v = value comparison cost

---

### 6. Enum (@enum types)

```hakorune
enum_eq(lhs, rhs, guard) {
  // 1. Check enum type
  if (lhs.enum_type() != rhs.enum_type()) {
    return 0
  }

  // 2. Check variant
  if (lhs.variant() != rhs.variant()) {
    return 0
  }

  // 3. Check payload (recursive)
  if (!guard.check(lhs, rhs)) { return 0 }

  local lhs_payload = lhs.payload()
  local rhs_payload = rhs.payload()
  local result = me.compare(lhs_payload, rhs_payload, guard)

  guard.uncheck(lhs, rhs)
  return result
}
```

---

### 7. User-Defined Equals

```hakorune
call_user_equals(lhs, rhs, guard) {
  // Lookup equals method via Resolver
  local type_id = lhs.type_id()
  local handle = Resolver.lookup(type_id, :equals, 1)

  if (handle == null) {
    // No user-defined equals → fallback to ptr_eq
    return ptr_eq(lhs, rhs)
  }

  // Call user equals with guard
  return ExecBox.call_by_handle(handle, [lhs, rhs], guard)
}
```

**Important**: User `equals` receives `guard` as implicit parameter

---

## 🧪 Golden Testing Strategy

### Goal: Rust-VM vs Hako-VM Parity

**Test Suite**: `tests/golden/op_eq/`

```
op_eq/
├── primitives.hako        # Int, Bool, Null
├── strings.hako           # String equality
├── arrays.hako            # Array equality
├── maps.hako              # Map equality
├── enums.hako             # @enum equality
├── recursion.hako         # Cyclic structures
└── user_defined.hako      # Custom equals methods
```

### Example Test: Cyclic Array

**File**: `tests/golden/op_eq/recursion.hako`

```hakorune
static box Main {
  main() {
    local arr1 = new ArrayBox()
    arr1.push(1)
    arr1.push(arr1)  // Self-reference

    local arr2 = new ArrayBox()
    arr2.push(1)
    arr2.push(arr2)  // Self-reference

    // Should NOT infinite loop
    if (arr1 == arr2) {
      return 1  // Expected: true (both self-referential)
    } else {
      return 0
    }
  }
}
```

**Verification**:
```bash
# Rust-VM
./hako --backend vm-rust tests/golden/op_eq/recursion.hako
# Output: 1

# Hako-VM
./hako --backend vm tests/golden/op_eq/recursion.hako
# Output: 1

# Expected: Both return 1 (no infinite loop)
```

---

## 📊 Implementation Timeline

### Week 5: Core Implementation

**Day 1-2**: NoOperatorGuard
- [ ] Guard data structure (MapBox)
- [ ] check/uncheck methods
- [ ] Unit tests

**Day 3-4**: Primitive Comparisons
- [ ] ptr_eq, int_eq, bool_eq, null_eq
- [ ] string_eq
- [ ] Unit tests

**Day 5-7**: Recursive Comparisons
- [ ] array_eq (with guard)
- [ ] map_eq (with guard)
- [ ] enum_eq (with guard)
- [ ] Integration tests

### Week 6: User-Defined + Golden Tests

**Day 1-2**: User-Defined Equals
- [ ] Resolver integration
- [ ] call_user_equals implementation
- [ ] Custom equals examples

**Day 3-5**: Golden Tests
- [ ] 20+ test cases (primitives, arrays, maps, enums, recursion)
- [ ] Rust-VM vs Hako-VM comparison
- [ ] CI integration

**Day 6-7**: Performance Tuning
- [ ] Profile op_eq execution
- [ ] Optimize hot paths
- [ ] Benchmark: Hako-VM ≥ 70% of Rust-VM speed

---

## 🎯 Success Criteria

### Functional

- [ ] NoOperatorGuard prevents infinite recursion
- [ ] All comparison types implemented (8 types)
- [ ] Golden tests: 100% Rust-VM parity
- [ ] No crashes or hangs

### Performance

- [ ] Hako-VM op_eq ≥ 70% of Rust-VM speed
- [ ] No memory leaks (guard cleanup)
- [ ] Acceptable overhead (< 10% in non-equality operations)

### Quality

- [ ] Unit tests for each comparison type
- [ ] Integration tests for complex cases
- [ ] Documentation for user-defined equals
- [ ] Examples for common patterns

---

## 🚧 Migration Strategy

### Phase 1: Hakorune Implementation (Week 5)

**Keep Rust op_eq**: Still in use (default)
**Add Hako op_eq**: New implementation (opt-in)

```bash
# Use Rust op_eq (default)
./hako test.hako

# Use Hako op_eq (opt-in)
HAKO_USE_PURE_EQ=1 ./hako test.hako
```

### Phase 2: Golden Testing (Week 6)

**Run both implementations**:
```bash
for test in tests/golden/op_eq/*.hako; do
  # Rust op_eq
  ./hako --backend vm-rust "$test" > rust.txt

  # Hako op_eq
  HAKO_USE_PURE_EQ=1 ./hako --backend vm "$test" > hako.txt

  # Compare
  diff rust.txt hako.txt || echo "FAIL: $test"
done
```

### Phase 3: Switchover (End of Week 6)

**Make Hako op_eq default**:
```rust
// src/vm_ops/compare/mod.rs
pub fn op_eq(lhs: &Value, rhs: &Value) -> bool {
    if env::var("HAKO_USE_RUST_EQ").is_ok() {
        // Old Rust implementation (compat mode)
        rust_op_eq(lhs, rhs)
    } else {
        // New Hakorune implementation (default)
        call_hako_op_eq(lhs, rhs)
    }
}
```

### Phase 4: Rust Deprecation (Phase 20.6+)

**Remove Rust op_eq entirely**:
- Rust side only calls Hakorune
- No Rust equality logic
- "Rust=floor, Hakorune=house" ✅

---

## 🛡️ Edge Cases

### 1. Cyclic Structures

**Test**:
```hakorune
local arr = new ArrayBox()
arr.push(arr)  // Self-reference
arr == arr  // Should return 1 (no infinite loop)
```

**Solution**: NoOperatorGuard detects `(arr, arr)` already visiting

---

### 2. Mixed Types

**Test**:
```hakorune
1 == "1"  // Should return 0 (Int != String)
```

**Solution**: Type check before comparison

---

### 3. Null Handling

**Test**:
```hakorune
null == null  // Should return 1
null == 0     // Should return 0
```

**Solution**: Special-case null in comparison order

---

### 4. Floating Point (Future)

**Test**:
```hakorune
3.14 == 3.14  // Should return 1
NaN == NaN    // Should return 0 (IEEE 754)
```

**Solution**: Add float_eq with IEEE 754 rules (Phase 20.6+)

---

## 📚 Related Documents

- [STRATEGY_RECONCILIATION.md](STRATEGY_RECONCILIATION.md) - Why migrate op_eq?
- [PURE_HAKORUNE_ROADMAP.md](PURE_HAKORUNE_ROADMAP.md) - Overall plan
- [CHATGPT_PURE_HAKORUNE_STRATEGY.md](CHATGPT_PURE_HAKORUNE_STRATEGY.md) - Original guidance

---

**Status**: Design (Week 5-6 implementation)
**Owner**: ChatGPT (implementation), Claude (review)
**Timeline**: Week 5-6 of Phase 20.5
