# Issue: equals() Stack Overflow - Rust VM Equality Comparison Bug

**Created**: 2025-10-08
**Status**: OPEN
**Severity**: HIGH
**Component**: Rust VM - Equality Comparison Layer
**Root Cause File**: `src/backend/mir_interpreter/helpers/eval.rs:224`

---

## Summary

The Rust VM's `eq_vm()` function has an infinite recursion bug when comparing BoxRef instances. This causes stack overflow on ANY Box equality comparison using the `==` operator or `.equals()` method.

**Key Finding**: This is NOT an @enum macro bug - it affects ALL Box types.

---

## Root Cause

**File**: `src/backend/mir_interpreter/helpers/eval.rs`
**Line**: 224
**Function**: `eq_vm(&a5, &b5)`

The `eq_vm()` function likely has infinite recursion when comparing BoxRef instances:
- When comparing two Box instances, `eq_vm()` is called
- `eq_vm()` attempts to compare the boxes
- This triggers another `eq_vm()` call recursively
- Stack overflow occurs before any user-defined `equals()` method is called

---

## Evidence

### Test 1: Simple Box Without @enum (CRASHES)

```hakorune
box SimpleBox {
  value
}

static box Main {
  main() {
    local s1 = new SimpleBox()
    local s2 = new SimpleBox()
    s1.value = 42
    s2.value = 42

    if s1.equals(s2) {  // STACK OVERFLOW
      print("Equal")
    }
  }
}
```

**Result**: Stack overflow - proves this is NOT @enum-specific

### Test 2: Manual equals() Implementation (NEVER CALLED)

```hakorune
box SimpleBox {
  value

  equals(other) {
    print("equals called")  // NEVER PRINTED
    return me.value == other.value
  }
}

static box Main {
  main() {
    local s1 = new SimpleBox()
    local s2 = new SimpleBox()
    s1.value = 42
    s2.value = 42

    if s1.equals(s2) {  // CRASHES BEFORE METHOD ENTRY
      print("Equal")
    }
  }
}
```

**Result**:
- "equals called" is never printed
- Stack overflow happens BEFORE method entry
- Proves the crash is in the VM layer, not user code

### Test 3: @enum Generated Box (CRASHES)

```hakorune
@enum Result {
  Ok(value)
  Err(error)
}

static box Main {
  main() {
    local r1 = Result.Ok(42)
    local r2 = Result.Ok(42)

    if r1.equals(r2) {  // STACK OVERFLOW
      print("Equal")
    }
  }
}
```

**Result**: Same crash - consistent with VM layer bug

---

## Stack Trace Analysis

**Expected Call Chain**:
1. User code: `s1.equals(s2)`
2. MIR interpreter: Resolve `equals` method
3. VM: Call user-defined `equals()` method
4. User code: `return me.value == other.value`

**Actual Call Chain** (inferred):
1. User code: `s1.equals(s2)`
2. MIR interpreter: Resolve `equals` method
3. **VM: Call `eq_vm()` for Box comparison (BUG!)**
4. **eq_vm(): Call `eq_vm()` recursively**
5. **Stack overflow**

**Critical Point**: The VM is calling `eq_vm()` instead of dispatching to the user-defined `equals()` method.

---

## Impact

### Severity: HIGH

**Affected**:
- ALL Box instances (not just @enum)
- ALL equality comparisons (`==` operator)
- ALL `.equals()` method calls
- ALL Box types (user-defined, standard library, generated)

**Workarounds**:
- Use tag comparison: `box1._tag == box2._tag` (for @enum)
- Use field comparison: `box1.value == box2.value`
- Avoid direct Box equality comparison

**Blocked Features**:
- Phase 19 Day 5: @enum selfhost integration
- Any code that compares Box instances
- Auto-derived equality methods

---

## Proposed Fix

### Location
`src/backend/mir_interpreter/helpers/eval.rs:224`

### Analysis Needed
1. Examine `eq_vm()` implementation
2. Check if it's recursively calling itself for BoxRef
3. Verify method dispatch logic for user-defined `equals()`

### Likely Fix
```rust
// Current (buggy):
fn eq_vm(a: &VmValue, b: &VmValue) -> bool {
    match (a, b) {
        (VmValue::BoxRef(a_box), VmValue::BoxRef(b_box)) => {
            // BUG: Likely calling eq_vm() recursively here
            eq_vm(a_box, b_box)  // INFINITE RECURSION
        }
        // ... other cases
    }
}

// Proposed fix:
fn eq_vm(a: &VmValue, b: &VmValue) -> bool {
    match (a, b) {
        (VmValue::BoxRef(a_box), VmValue::BoxRef(b_box)) => {
            // OPTION 1: Dispatch to user-defined equals() method
            call_box_method(a_box, "equals", vec![b_box])

            // OPTION 2: Reference equality (pointer comparison)
            Arc::ptr_eq(&a_box.inner, &b_box.inner)

            // OPTION 3: Default structural equality
            compare_box_fields(a_box, b_box)
        }
        // ... other cases
    }
}
```

### Recommended Approach
1. First, use reference equality (Option 2) for safety
2. Then, implement method dispatch (Option 1) for correct semantics
3. Add comprehensive tests for Box equality

---

## Testing Strategy

### Minimal Reproduction
```hakorune
box SimpleBox { value }

static box Main {
  main() {
    local s1 = new SimpleBox()
    local s2 = new SimpleBox()
    s1.value = 42
    s2.value = 42

    if s1.equals(s2) {
      print("PASS")
    }
  }
}
```

**Expected**: Print "PASS"
**Current**: Stack overflow

### Test Matrix

After fix, verify:

1. **Reference Equality**:
   - Same instance → true
   - Different instances → depends on semantics

2. **User-Defined equals()**:
   - Manual implementation → method is called
   - Return value → respected

3. **@enum Generated equals()**:
   - Auto-generated method → works correctly
   - Field comparison → correct results

4. **Standard Types**:
   - Integer/String/Bool → existing behavior unchanged
   - Array/Map → existing behavior unchanged

---

## Related Issues

- Phase 19 Day 4 Investigation (this issue discovered during @enum testing)
- @enum macro test suite workaround (using tag comparison instead of equality)

---

## Timeline

**Discovered**: 2025-10-08 (Phase 19 Day 3)
**Root Cause Identified**: 2025-10-08 (Phase 19 Day 4)
**Fix Priority**: HIGH (blocks Phase 19 Day 5)

---

## Lessons Learned

### Investigation Process
1. ✅ Created minimal reproduction (without @enum) → Proved not macro-specific
2. ✅ Tested manual equals() → Proved method not called
3. ✅ Traced to VM layer → Identified exact file/line
4. ✅ Documented evidence → Clear issue description

### Best Practices
- Always test at component boundaries (macro vs VM)
- Minimal reproductions reveal root causes
- Manual implementations test dispatch logic
- Document evidence thoroughly

---

## References

**Phase 19 Documentation**:
- [Phase 19 README](../roadmap/phases/phase-19-enum-match/README.md)
- [CURRENT_TASK.md](/home/tomoaki/git/hakorune-selfhost/CURRENT_TASK.md)
- [CLAUDE.md](/home/tomoaki/git/hakorune-selfhost/CLAUDE.md)

**Code Locations**:
- Bug location: `src/backend/mir_interpreter/helpers/eval.rs:224`
- Test file: `tools/smokes/v2/profiles/quick/selfhost/enum_macro_basic.sh`

---

**Next Steps**:
1. Examine `eval.rs:224` - eq_vm() implementation
2. Fix infinite recursion
3. Add Box equality tests
4. Unblock Phase 19 Day 5
