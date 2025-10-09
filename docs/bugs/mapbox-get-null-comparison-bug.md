# MapBox.get() Null Comparison Bug - Investigation Report

**Date**: 2025-10-09
**Status**: FIXED (default behavior updated)
**Severity**: HIGH (affects all comparison operations with MapBox)

## Executive Summary

**Root Cause**: `MapBox.get()` returns `StringBox("Key not found: <key>")` instead of `null` when a key doesn't exist, causing all comparisons with 0 to fail.

**Impact**: Any code using the pattern `if map.get(key) == null { ... }` or `if value != 0 { ... }` after `value = map.get(key)` will fail.

**Fix**: As of 2025‑10‑09, `MapBox.get(missing)` returns `null` by default. Prefer direct `v = map.get(key); if v == null { ... }` and remove string‑error checks.

## Problem Description

### Original Symptom

Test 3 in `test_phase2_day5.hako` exhibited strange behavior:

```hako
local result3 = HakoruneVmCore.run(mir3)
print("Test 3: load from uninitialized mem[99] → " + StringHelpers.int_to_str(result3))

if result3 != 0 {
  print("[FAIL] Test 3: expected 0, got " + StringHelpers.int_to_str(result3))
  return 1
}
```

**Output**:
```
Test 3: load from uninitialized mem[99] → 0
[FAIL] Test 3: expected 0, got 0
```

The print statement shows "expected 0, got 0", yet the comparison `result3 != 0` evaluated to `true`!

## Investigation Process

### Phase 1: Basic Comparison Tests ✅

Created `test_compare_bug.hako` to test basic comparisons:
- Direct literal comparison (0 != 0): **PASS**
- Variable assignment comparison: **PASS**
- Comparison with string conversion: **PASS**
- All basic comparisons work correctly

**Conclusion**: The comparison operator itself is not broken.

### Phase 2: VM Return Value Tests ❌

Created `test_vm_return_compare.hako` to test VM-returned values:

```hako
// Test 2.1: VM returns const 0
local mir1 = r#"{"functions":[{"name":"test","blocks":[{"id":0,"instructions":[{"op":"const","dst":1,"value":{"type":"i64","value":0}},{"op":"ret","value":1}],"terminator":{"op":"ret","value":1}}]}]}"#
local result1 = HakoruneVmCore.run(mir1)
if result1 != 0 {
  print("[FAIL]")
}
```
**Result**: PASS

```hako
// Test 2.2: VM returns 0 from Load (uninitialized)
local mir2 = r#"{"functions":[{"name":"test","blocks":[{"id":0,"instructions":[{"op":"load","dst":1,"ptr":99},{"op":"ret","value":1}],"terminator":{"op":"ret","value":1}}]}]}"#
local result2 = HakoruneVmCore.run(mir2)
if result2 != 0 {
  print("[FAIL]")
}
```
**Result**: **FAIL** - This is the bug!

**Conclusion**: Bug occurs specifically when value comes from a **Load instruction** returning 0 from uninitialized memory.

### Phase 3: Null vs Zero Comparison ⚠️

Created `test_null_vs_zero.hako`:

```hako
local a = null
local b = 0

print("a (null) value: " + StringHelpers.int_to_str(a))  // prints "0"
print("b (0) value: " + StringHelpers.int_to_str(b))     // prints "0"

if a != 0 {
  print("a (null) != 0 is TRUE")  // ← THIS EXECUTES
}

if b != 0 {
  print("b (0) != 0 is TRUE")
} else {
  print("b (0) != 0 is FALSE")    // ← THIS EXECUTES
}
```

**Result**: `null != 0` returns `true`, while `0 != 0` returns `false`!

**Conclusion**: null and 0 are different values in comparisons, even though they both print as "0".

### Phase 4: MapBox.get() Behavior Investigation 🎯

Created `test_mapbox_get_behavior.hako`:

```hako
local map = new MapBox()
local result = map.get("nonexistent")

print("Direct print: " + result)
// Output: Direct print: Key not found: nonexistent

if result == null {
  print("result == null: TRUE")
} else {
  print("result == null: FALSE")  // ← THIS EXECUTES
}

if result != 0 {
  print("result != 0: TRUE")      // ← THIS EXECUTES (THE BUG!)
}
```

**Root Cause Found**: `MapBox.get()` returns `StringBox("Key not found: <key>")` instead of null!

### Rust Implementation Verification

Checked `/home/tomoaki/git/hakorune-selfhost/src/boxes/map_box.rs` line 139-188:

```rust
pub fn get(&self, key: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
    let key_str = key.to_string_box().value;
    let guard = self.data.read().unwrap();
    match guard.get(&key_str) {
        Some(value) => {
            value.clone_box()
        }
        None => {
            // Returns error message instead of null!
            Box::new(StringBox::new(&format!("Key not found: {}", key_str)))
        }
    }
}
```

**Confirmation**: MapBox.get() intentionally returns an error message StringBox instead of null.

## Affected Code Locations

Found **5 instances** of the buggy pattern:

1. **terminator_handler.hako:86-88** (ret instruction)
   ```hako
   local val = regs.get(StringHelpers.int_to_str(val_id))
   if val == null {
     val = 0
   }
   ```

2. **terminator_handler.hako:140-142** (branch instruction)
   ```hako
   local cond_val = regs.get(StringHelpers.int_to_str(cond_id))
   if cond_val == null {
     cond_val = 0
   }
   ```

3. **phi_handler.hako:113-115** (phi instruction)
   ```hako
   local value = regs.get(StringHelpers.int_to_str(value_id))
   if value == null {
     value = 0
   }
   ```

4. **value_manager.hako:13-16** (register get)
   ```hako
   local val = regs.get(key)
   if val == null {
     return 0
   }
   ```

5. **load_handler.hako:32-35** (memory load)
   ```hako
   local mem_value = mem.get(ptr_key)
   if mem_value != null {
     value = mem_value
   }
   ```

## Fix Applied

### Pattern: Before (Buggy)
```hako
local value = map.get(key)
if value == null {
  value = 0
}
```

### Pattern: After (Fixed)
```hako
// Fix: MapBox.get() returns StringBox("Key not found") instead of null
// Must use has() to check existence first
local value = 0
if map.has(key) {
  value = map.get(key)
}
```

### Files Modified

1. `/home/tomoaki/git/hakorune-selfhost/apps/selfhost/hakorune-vm/terminator_handler.hako`
   - Fixed `_handle_ret()` (line 84-92)
   - Fixed `_handle_branch()` (line 139-146)

2. `/home/tomoaki/git/hakorune-selfhost/apps/selfhost/hakorune-vm/phi_handler.hako`
   - Fixed phi value loading (line 112-118)

3. `/home/tomoaki/git/hakorune-selfhost/apps/selfhost/hakorune-vm/value_manager.hako`
   - Fixed `get()` method (line 11-19)

4. `/home/tomoaki/git/hakorune-selfhost/apps/selfhost/hakorune-vm/load_handler.hako`
   - Fixed memory load (line 27-35)

## Test Results

### Before Fix
```
Test 3: load from uninitialized mem[99] → 0
[FAIL] Test 3: expected 0, got 0
```

### After Fix
```
Test 3: load from uninitialized mem[99] → 0
✅ All Phase 2 Day 5 tests PASSED!
```

## Minimal Reproduction Case

```hako
static box Main {
  main() {
    local map = new MapBox()
    local result = map.get("missing")

    // This FAILS before fix
    if result != 0 {
      print("BUG: result is StringBox('Key not found'), not 0")
      return 1
    }

    // Correct pattern (after fix)
    local value = 0
    if map.has("missing") {
      value = map.get("missing")
    }

    if value != 0 {
      print("This will NOT execute")
    }

    return 0
  }
}
```

## Lessons Learned

### For Future Development

1. **Never assume MapBox.get() returns null** - always use `has()` first
2. **Test edge cases explicitly** - uninitialized memory/registers are critical test cases
3. **Verify assumptions about built-in types** - even basic operations may have unexpected behavior
4. **Create minimal reproduction cases early** - helps isolate the root cause quickly

### Design Questions for Future

Should MapBox.get() behavior be changed? Two options:

**Option A**: Return null for missing keys (breaking change)
- Pros: More intuitive, matches common dictionary behavior
- Cons: Breaks existing code that may rely on error messages

**Option B**: Keep current behavior (return error message)
- Pros: No breaking changes, explicit error messages
- Cons: Requires developers to always use has() pattern

**Recommendation (updated)**: Document the default `null` behavior in MapBox API docs; create a linter to detect legacy patterns that look for `"Key not found:"` strings or rely on `has()+get()` where a single `get()+null` suffices.

## Test Files Created

1. `/home/tomoaki/git/hakorune-selfhost/apps/selfhost/hakorune-vm/tests/test_compare_bug.hako`
   - Phase 1: Basic comparison tests

2. `/home/tomoaki/git/hakorune-selfhost/apps/selfhost/hakorune-vm/tests/test_vm_return_compare.hako`
   - Phase 2: VM return value comparison tests

3. `/home/tomoaki/git/hakorune-selfhost/apps/selfhost/hakorune-vm/tests/test_null_vs_zero.hako`
   - Phase 3: null vs 0 comparison investigation

4. `/home/tomoaki/git/hakorune-selfhost/apps/selfhost/hakorune-vm/tests/test_mapbox_get_behavior.hako`
   - Phase 4: MapBox.get() behavior investigation

All test files can be run with:
```bash
HAKO_ALLOW_USING_FILE=1 HAKO_USING_PROFILE=dev NYASH_USING_AST=1 NYASH_DISABLE_PLUGINS=1 ./target/release/hako <test_file>
```

## Summary

This bug was caused by a fundamental misunderstanding of MapBox.get() behavior. The Rust implementation returns an error message StringBox instead of null for missing keys, which behaves differently in comparisons.

The fix is simple but requires vigilance: **always use `map.has(key)` before `map.get(key)` when checking for existence**.

All affected files have been fixed and tested. The original Test 3 now passes correctly.
