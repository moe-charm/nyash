# Issue: LLVM PHI Value Resolution Bug - Silent Exception Masking

**Created**: 2025-10-08
**Updated**: 2025-10-08
**Status**: 🔍 INVESTIGATING
**Severity**: MEDIUM (functionally correct but generates suboptimal IR)
**Component**: LLVM Backend - PHI Resolution
**Root Cause File**: `src/llvm_py/instructions/externcall.py:63-77`

---

## Summary

When compiling Hakorune code with PHI nodes, the `op_eq` implementation generates incorrect LLVM IR that uses constant zeros instead of PHI values. **However, the final result is correct** due to other optimizations or coincidence.

**Key Finding**: Silent exception swallowing at `externcall.py:67` masks the real error.

---

## Evidence

### Test Case: PHI After Branch

**File**: `/tmp/test_op_eq_return.nyash`
```hakorune
static box Main {
  main() {
    local a = 42
    local b = 42
    local c = 10
    if a == b {
      if a == c {  // ← Second comparison uses PHI values
        return 1
      } else {
        return 0  // ← Correct path
      }
    } else {
      return 2
    }
  }
}
```

### MIR (Correct)

```mir
bb4:
    0: %19 = phi [%2, bb3]  // %19 = 10
    1: %20 = phi [%0, bb3]  // %20 = 42
    2: %25 = copy %20
    3: %26 = copy %19
    4: %28 = copy %25
    5: %29 = copy %26
    6: %30 = copy %28
    7: %31 = copy %29
    8: %27 = call_extern nyrt.ops.op_eq(%30, %31)  // Should use %30 and %31
```

### Generated LLVM IR (Bug)

```llvm
bb4:
  %"phi_21" = phi  i64 [42, %"bb3"]
  %"phi_18" = phi  i64 [10, %"bb3"]
  %"op_eq_cmp.1" = icmp eq i64 0, 0  ; ← Should be: icmp eq i64 %phi_21, %phi_18
```

**Problem**: PHI values defined but not used, comparing `0 == 0` instead.

### Expected LLVM IR (Correct)

```llvm
bb4:
  %"phi_21" = phi  i64 [42, %"bb3"]
  %"phi_18" = phi  i64 [10, %"bb3"]
  ; After copy chain resolution: %30 → %phi_21, %31 → %phi_18
  %"op_eq_cmp.1" = icmp eq i64 %"phi_21", %"phi_18"
```

---

## Root Cause Analysis

### The Bug Location

**File**: `src/llvm_py/instructions/externcall.py:63-77`

```python
def _resolve_i64(vid: int):
    if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
        try:
            return PhiDispatchPoint.resolve_i64(builder, resolver, int(vid), builder.block, preds, block_end_values, vmap, bb_map)
        except Exception:
            pass  # ← CULPRIT: Silent exception swallowing!
    v = vmap.get(vid)
    if v is None:
        return ir.Constant(i64, 0)  # ← Returns 0 when vmap lookup fails
```

### The Problem Flow

1. **Step 1**: `_resolve_i64(30)` called for first argument
2. **Step 2**: `PhiDispatchPoint.resolve_i64()` throws exception (unknown reason)
3. **Step 3**: Exception caught at line 67, silently ignored
4. **Step 4**: Fallback to `vmap.get(30)` at line 69
5. **Step 5**: Returns `None` (value not in local vmap)
6. **Step 6**: Line 71 returns `ir.Constant(i64, 0)` → **BUG!**

### Why vmap.get(vid) Returns None

**vmap Scope Mismatch**:

- **Global vmap** (`owner.vmap`): Contains PHI values and copy chain values
- **Local vmap** (`vmap_cur`): Per-block vmap, PHI values copied in
- **Sync timing**: Copy chain values (%30, %31) stored in global vmap
- **Issue**: `op_eq` receives local vmap which may not have copy chain values yet

**Key Files**:
- `src/llvm_py/llvm_builder.py:263-280` - Per-block vmap creation
- `src/llvm_py/llvm_builder.py:330-342` - vmap sync after each instruction
- `src/llvm_py/builders/instruction_lower.py:50` - vmap context selection

### Why Result is Still Correct

Despite the bug, exit code is `0` (correct):
- First comparison (`bb3`): `42 == 42` → true (correct)
- Second comparison (`bb4`): `0 == 0` → true (wrong reason, correct result!)
- Takes correct path: bb4 → bb8 → ret 0

**This is a ticking time bomb**: Different test values would fail.

---

## Impact

### Severity: MEDIUM

**Why not HIGH**:
- Final result happens to be correct for current test
- Doesn't cause crash or wrong exit code
- Limited to PHI + copy chain scenarios

**Why not LOW**:
- Generates incorrect IR (semantic bug)
- Silent exception masking (hides real problem)
- Will fail with different test values
- Affects all extern calls using `_resolve_i64()` helper

**Affected Scenarios**:
- ✅ Simple comparisons (no PHI): Work correctly
- ❌ Comparisons after branch (with PHI): Wrong IR, correct result (coincidence)
- ❓ Complex PHI scenarios: Untested, likely broken

**NOT Affected**:
- VM backend (completely separate code path)
- Compare instruction (doesn't use externcall.py)
- LLVM AOT (different lowering path)

---

## Proposed Fixes

### Option 1: Remove Silent Exception Swallowing ⭐ **RECOMMENDED**

**Goal**: See the **actual error** being hidden

**File**: `src/llvm_py/instructions/externcall.py:63-77`

```python
def _resolve_i64(vid: int):
    if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
        # DON'T swallow exceptions - let them propagate
        return PhiDispatchPoint.resolve_i64(builder, resolver, int(vid), builder.block, preds, block_end_values, vmap, bb_map)

    v = vmap.get(vid)
    if v is None:
        # Add diagnostic before returning 0
        import sys
        print(f"[op_eq] WARNING: vmap.get({vid}) returned None in block {builder.block.name}", file=sys.stderr)
        print(f"[op_eq] Available vmap keys: {list(vmap.keys())}", file=sys.stderr)
        return ir.Constant(i64, 0)

    # ... rest unchanged
```

**Pros**:
- Reveals the real error
- No behavior change (still returns 0 on failure)
- Diagnostic output helps debugging

**Cons**:
- Will expose exception (breaking change if code depends on swallowing)

---

### Option 2: Add Global vmap Fallback

**Goal**: Try global vmap if local vmap lookup fails

```python
def _resolve_i64(vid: int):
    if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
        try:
            return PhiDispatchPoint.resolve_i64(builder, resolver, int(vid), builder.block, preds, block_end_values, vmap, bb_map)
        except Exception as e:
            import sys
            print(f"[op_eq] PhiDispatchPoint failed for vid={vid}: {e}", file=sys.stderr)

    # Try local vmap first
    v = vmap.get(vid)

    # Fallback to global vmap if available
    if v is None and hasattr(module, '_global_vmap'):
        v = module._global_vmap.get(vid)
        if v is not None:
            import sys
            print(f"[op_eq] Found vid={vid} in global vmap (not in local)", file=sys.stderr)

    if v is None:
        return ir.Constant(i64, 0)

    # ... type coercion unchanged
```

**Pros**:
- Might fix the bug if global vmap has the values
- Preserves exception handling

**Cons**:
- Doesn't address root cause
- May hide real design issue

---

### Option 3: Debug Logging (Investigation)

**Goal**: Understand what's happening before trying to fix

```python
def _resolve_i64(vid: int):
    import sys
    print(f"[op_eq] Resolving vid={vid} in block={builder.block.name}", file=sys.stderr)
    print(f"[op_eq] Local vmap keys: {list(vmap.keys())}", file=sys.stderr)

    if resolver is not None and preds is not None and block_end_values is not None and bb_map is not None:
        try:
            result = PhiDispatchPoint.resolve_i64(builder, resolver, int(vid), builder.block, preds, block_end_values, vmap, bb_map)
            print(f"[op_eq] PhiDispatchPoint SUCCESS: vid={vid} → {result}", file=sys.stderr)
            return result
        except Exception as e:
            import traceback
            print(f"[op_eq] PhiDispatchPoint FAILED: vid={vid}", file=sys.stderr)
            print(f"[op_eq] Exception: {e}", file=sys.stderr)
            traceback.print_exc(file=sys.stderr)

    v = vmap.get(vid)
    print(f"[op_eq] vmap.get({vid}) = {v}", file=sys.stderr)

    if v is None:
        print(f"[op_eq] WARNING: Returning Constant(0) for vid={vid}", file=sys.stderr)
        return ir.Constant(i64, 0)

    # ... rest unchanged
```

Run with test case to see detailed trace.

---

## Related Code Locations

**Bug Site**:
- `src/llvm_py/instructions/externcall.py:63-77` - `_resolve_i64()` helper

**PHI Resolution System**:
- `src/llvm_py/dispatch/phi_dispatch.py:118-187` - `PhiDispatchPoint.resolve_i64()`
- `src/llvm_py/llvm_builder.py:263-342` - Per-block vmap management
- `src/llvm_py/instructions/phi.py:123` - PHI value storage
- `src/llvm_py/instructions/copy.py:43-46` - Copy instruction lowering

**Context Passing**:
- `src/llvm_py/builders/instruction_lower.py:50` - vmap context selection
- `src/llvm_py/builders/instruction_lower.py:156-157` - externcall lowering

---

## Testing Strategy

### Reproduction

```bash
# Generate IR with bug
NYASH_LLVM_USE_HARNESS=1 ./target/release/hakorune --backend llvm /tmp/test_op_eq_return.nyash

# Check generated IR
cat /tmp/debug_ir.ll | grep -A 10 "bb4:"
# Look for: icmp eq i64 0, 0  (BUG)
# Should be: icmp eq i64 %phi_21, %phi_18
```

### Test Matrix (After Fix)

1. **Simple comparison** (no PHI):
   ```hakorune
   if 42 == 42 { return 0 }
   ```
   Expected: `icmp eq i64 42, 42` ✅

2. **Comparison after branch** (with PHI):
   ```hakorune
   if a == b {
     if a == c { ... }  // Uses PHI values
   }
   ```
   Expected: `icmp eq i64 %phi_X, %phi_Y` ❌ Currently broken

3. **Deep copy chain**:
   ```hakorune
   local x = a
   local y = x
   local z = y
   if z == b { ... }
   ```
   Expected: Resolve through copy chain ❓ Untested

---

## Next Steps

### Immediate (Phase 19 Day 6)
1. ✅ **Document bug** (this file)
2. ⏳ **Apply Option 1** (remove silent exception)
3. ⏳ **Run test with diagnostics** to see real error
4. ⏳ **Determine root cause** from exception message
5. ⏳ **Implement proper fix** based on findings

### Short-term (Phase 20)
- Add comprehensive PHI resolution tests
- Refactor `_resolve_i64()` to use PhiDispatchPoint consistently
- Consider unifying vmap management (global vs local)

### Long-term
- Full vmap architecture review
- PhiDispatchPoint API simplification
- LLVM IR validation pass (detect `icmp eq i64 0, 0` patterns)

---

## Related Issues

- Phase 19 Day 5: ExternCall→Call+Callee::Extern unification
- auto-generated-equals-bug.md: RESOLVED (separate issue)
- print() string output bug: Separate issue, unrelated

---

## Timeline

**Discovered**: 2025-10-08 (Phase 19 Day 5)
**Root Cause Identified**: 2025-10-08 (task agent investigation)
**Status**: 🔍 INVESTIGATING

---

## References

**Investigation Report**: Provided by task agent 2025-10-08
**Test Case**: `/tmp/test_op_eq_return.nyash`
**Generated IR**: `/tmp/debug_ir.ll` (lines 50-56 show the bug)

---

**Note**: This bug is **not blocking** current development but should be fixed before Phase 20+ to prevent future issues with complex PHI scenarios.
