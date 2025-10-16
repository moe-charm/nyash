# Phase 31: Map.values() Returns Void Investigation Report

**Date**: 2025-10-16
**Problem**: `map_values_array_element_vm` fails with "Type error: nyrt.array.size expects ArrayBox"
**Symptom**: Map.values() returns Void instead of ArrayBox (Stage-2 array)

---

## Executive Summary

**Root Cause Found**: `Callee::Extern()` is disabled in non-legacy builds.

The Builder correctly:
1. Lowers `Map.values()` → `Extern("nyrt.map.values")`
2. Annotates the result as `Box("ArrayBox")`
3. Emits `call_extern nyrt.map.values(%11)`

But the VM rejects it:
```
Invalid instruction: extern calls disabled (legacy-only)
```

This happens because `handle_callee_extern()` in `function.rs:169-194` is **entirely gated behind `#[cfg(feature = "legacy-boxes")]`**.

---

## Detailed Analysis

### 1. Execution Path Trace

#### Builder Phase (CORRECT)

**File**: `src/mir/builder/lowering/mod.rs:42`
```rust
("values", 0) => Some(LoweredExternSpec {
    extern_name: "nyrt.map.values",
    prepend_recv: true
}),
```

**File**: `src/mir/builder/method_call_handlers.rs:148-153`
```rust
"nyrt.map.values" | "nyrt.map.keys" => {
    builder
        .value_types
        .insert(dst, crate::mir::MirType::Box("ArrayBox".into()));
    builder.origin_register(dst, "ArrayBox".to_string());
}
```

**MIR Output**:
```mir
13: %9: Box("ArrayBox") = call_extern nyrt.map.values(%11)
```

✅ **Builder is 100% correct!**

---

#### VM Phase (BROKEN)

**File**: `src/backend/mir_interpreter/handlers/calls/function.rs:169-194`
```rust
/// Handle Extern callee: emit trace then dispatch to externs.
pub(crate) fn handle_callee_extern(
    &mut self,
    extern_name: &str,
    args: &[ValueId],
) -> Result<VMValue, VMError> {
    let label = format!("Extern:{}", extern_name);
    self.emit_call_trace_label(&label, args.len(), None);

    // ... ffi.dynamic handling ...

    #[cfg(feature = "legacy-boxes")]
    { self.execute_extern_function(extern_name, args) }
    #[cfg(not(feature = "legacy-boxes"))]
    { Err(VMError::InvalidInstruction(
        crate::backend::mir_interpreter::diagnostics::DIAG_EXTERN_DISABLED.into()
    )) }
}
```

❌ **Feature gate blocks ALL Extern callees in plugin builds!**

---

### 2. Void Injection Point

**Execution Flow**:
```
MIR: call_extern nyrt.map.values(%11)
  ↓
VM: handle_callee_extern("nyrt.map.values", args)
  ↓
VM: #[cfg(not(feature = "legacy-boxes"))]
  ↓
VM: Err("extern calls disabled (legacy-only)")
  ↓
??? (Error should propagate, but we see Void)
```

**Mystery**: The error message appears in stderr, but execution continues and returns Void.

**Hypothesis**: There's an error-swallowing fallback somewhere between the Call instruction handler and the final result.

---

### 3. Extern Adapter Status

The extern adapter **exists and works correctly**!

**File**: `src/backend/mir_interpreter/extern_adapter.rs:106-121`
```rust
pub fn try_call(iface: &str, method: &str, loaded_args: &[VMValue])
    -> Option<Result<VMValue, VMError>>
{
    let key = (iface.to_string(), method.to_string());
    if let Some(h) = adapter().handlers.get(&key) {
        return Some(h(loaded_args));
    }
    // Fallback: consult externs registry
    if extreg::registry().get(iface, method).is_some() {
        return Some(Err(VMError::InvalidInstruction(format!(
            "Extern {}.{} has spec but no handler",
            iface, method
        ))));
    }
    None
}
```

**File**: `src/backend/mir_interpreter/extern_adapter/extern_map.rs:102-145`
```rust
// nyrt.map.values(recv:Map) -> Array
map.insert(("nyrt.map".into(), "values".into()), |args: &[VMValue]| {
    if args.is_empty() {
        return Err(VMError::InvalidInstruction(
            "nyrt.map.values requires receiver".into()
        ));
    }
    match &args[0] {
        VMValue::BoxRef(b) => {
            // Host slot call (plugins)
            if let Some((_route, value)) =
                host_slot::try_invoke_arc(b, MAP_HOST_ROUTES, "values", &args[1..])
            {
                if std::env::var("NYASH_DEBUG_MAP_VALUES").ok().as_deref() == Some("1") {
                    eprintln!("[extern_map] host values -> {:?}", value);
                }
                return Ok(value);
            }
            // Plugin fallback
            if let Some(plugin_box) = b.as_any().downcast_ref::<PluginBoxV2>() {
                let out = plugin_host_box::invoke_instance_method(
                    "MapBox",
                    "values",
                    plugin_box.inner.instance_id,
                    &[],
                );
                let result = match out {
                    Ok(Some(ret)) => Ok(VMValue::from_nyash_box(ret)),
                    Ok(None) => Ok(VMValue::Void),  // ← SOURCE OF VOID!
                    Err(e) => Err(VMError::InvalidInstruction(format!(
                        "Plugin method MapBox.values failed: {:?}",
                        e
                    ))),
                };
                // ... debug logging ...
                return result;
            }
            // Legacy boxes fallback (disabled in plugin builds)
            Ok(VMValue::Void)  // ← FINAL FALLBACK
        }
        _ => Err(VMError::TypeError("nyrt.map.values expects MapBox".into())),
    }
});
```

**Important**: The extern adapter **is never called** because `handle_callee_extern()` rejects it before reaching `extern_adapter::try_call()`.

---

### 4. Comparison with Map.size() (Working)

**Map.size()** works because it uses a **different path**!

**File**: `src/mir/builder/normalize/map_length.rs:13-19`
```rust
pub fn normalize_map_length_call(
    builder: &mut MirBuilder,
    callee: &mut Callee,
    args: &mut Vec<ValueId>,
) -> bool {
    super::normalize_length_call(builder, callee, args, "MapBox", "nyrt.map.size")
}
```

**Key Difference**:
- `Map.size()` is **normalized** during MIR optimization phase
- The optimizer may rewrite `Extern("nyrt.map.size")` → `Method("size")` or use a different route
- `Map.values()` has **no normalization** - stays as `Extern("nyrt.map.values")`

**Verification Needed**: Check if optimizer rewrites Extern callees.

---

## Root Cause Chain

```
1. Builder: Map.values() → Extern("nyrt.map.values")  ✅ CORRECT
   ↓
2. Optimizer: No rewrite for Map.values()  ✅ CORRECT
   ↓
3. VM: Call instruction with Callee::Extern
   ↓
4. VM: Dispatch to handle_callee_extern()
   ↓
5. VM: #[cfg(not(feature = "legacy-boxes"))]  ❌ REJECTS
   ↓
6. VM: Returns Err("extern calls disabled")
   ↓
7. ??? Error is swallowed, returns Void  ❌ BUG
```

---

## Evidence

### Test Output
```bash
$ NYASH_DEBUG_MAP_VALUES=1 bash tools/smokes/v2/profiles/plugins/map_values_array_element_vm.sh
[map-values-debug] extern callee=nyrt.array.size dst=Some(ValueId(15))
[map-values-debug] extern callee=nyrt.array.size dst=Some(ValueId(14))
Invalid instruction: extern calls disabled (legacy-only)
[FAIL] expected VSZ:1 and E0SZ:1
```

**Analysis**:
- Debug trace shows `extern callee=nyrt.array.size` (from Map.values() result)
- Error message appears: "extern calls disabled"
- Test fails with Void (no VSZ/E0SZ output)

---

## Solution

### Option 1: Enable Extern Calls in Plugin Builds (Recommended)

**File**: `src/backend/mir_interpreter/handlers/calls/function.rs:189-194`

**Before**:
```rust
#[cfg(feature = "legacy-boxes")]
{ self.execute_extern_function(extern_name, args) }
#[cfg(not(feature = "legacy-boxes"))]
{ Err(VMError::InvalidInstruction(
    crate::backend::mir_interpreter::diagnostics::DIAG_EXTERN_DISABLED.into()
)) }
```

**After**:
```rust
// Load args and dispatch to extern adapter
let mut loaded: Vec<VMValue> = Vec::with_capacity(args.len());
for a in args { loaded.push(self.reg_load(*a)?); }

// Try extern adapter first (works in all builds)
if let Some((iface, method)) = extern_name.rsplit_once('.') {
    if let Some(r) = crate::backend::mir_interpreter::extern_adapter::try_call(
        iface, method, &loaded
    ) {
        return r;
    }
}

// Legacy-only fallback
#[cfg(feature = "legacy-boxes")]
{ self.execute_extern_function(extern_name, args) }
#[cfg(not(feature = "legacy-boxes"))]
{ Err(VMError::InvalidInstruction(format!(
    "Unknown extern: {}", extern_name
))) }
```

**Impact**:
- ✅ Enables all `nyrt.*` externs in plugin builds
- ✅ Preserves legacy extern handling
- ✅ No MIR changes required
- ✅ Minimal code changes (~10 lines)

---

### Option 2: Rewrite Extern → BoxCall in Optimizer

**Pros**:
- Keeps extern handling in legacy-only code
- Uses existing BoxCall infrastructure

**Cons**:
- ❌ Requires optimizer changes (complex)
- ❌ Breaks Builder/VM separation
- ❌ May conflict with future extern optimizations

**Not recommended**.

---

## Recommendations

1. **Immediate Fix**: Implement Option 1 (enable extern adapter in plugin builds)
2. **Verify**: Run `map_values_array_element_vm` test after fix
3. **Regression Test**: Add test for Extern callees in plugin builds
4. **Documentation**: Update CLAUDE.md with Extern vs BoxCall distinction
5. **Future Work**: Consider unifying BoxCall/Extern paths (Phase 32?)

---

## Related Files

### Builder
- `src/mir/builder/lowering/mod.rs:42` - Map.values() lowering spec
- `src/mir/builder/method_call_handlers.rs:148` - ArrayBox annotation

### VM
- `src/backend/mir_interpreter/handlers/calls/function.rs:169` - handle_callee_extern() ❌
- `src/backend/mir_interpreter/extern_adapter.rs:106` - try_call() ✅
- `src/backend/mir_interpreter/extern_adapter/extern_map.rs:102` - Map.values() handler ✅

### Tests
- `tools/smokes/v2/profiles/plugins/map_values_array_element_vm.sh` - Failing test

---

## Next Steps

1. **Fix**: Add extern adapter call to `handle_callee_extern()`
2. **Test**: `bash tools/smokes/v2/profiles/plugins/map_values_array_element_vm.sh`
3. **Verify**: All Map methods (size/keys/values) work in plugin builds
4. **Commit**: "fix(vm): Enable Extern callees in plugin builds via extern_adapter"

---

## Appendix: MIR Dump

```mir
define i64 @Main.main(box<Main> %0) effects(read) {
bb3:
    0: %1 = new ArrayBox() .auto_birth(ArrayBox.birth/0)
    1: %2 = const 7
    2: %0 = copy %1
    3: %3 = call %0.push[#3](%2)
    4: %4 = new MapBox() .auto_birth(MapBox.birth/0)
    5: %5 = const "x"
    6: %7 = copy %4
    7: %8 = copy %5
    8: %9 = copy %1
    9: %1 = copy %7
   10: %6 = call %1.set(%8, %9)
   11: %11 = const "nyrt.map.values"
   12: %12 = copy %7
   13: %10 = call_extern nyrt.map.values(%12)  ← CORRECT!
   14: %13 = const "VSZ:"
   15: %14 = copy %10
   16: %16 = copy %14
   17: %17 = copy %16
   18: %15 = call_extern nyrt.array.size(%17)  ← %10 should be ArrayBox
   ...
}

Type annotations:
   13: %9: Box("ArrayBox") = call_extern nyrt.map.values(%11)  ✅
```

**Note**: MIR is 100% correct. The problem is entirely in the VM execution phase.
