# methodRef Scalability: Quick Summary

## TL;DR

**Question**: If every Box needs methodRef implemented individually, doesn't that scale poorly?

**Answer**: **Hakorune already has the solution!** methodRef is NOT Box-specific logic — it's a thin wrapper calling universal functions.

## Key Facts

### Current State
- ✅ **ArrayBox** has methodRef (6 lines of code)
- ❌ **MapBox**, **StringBox**, **UserBoxes** don't have methodRef
- ✅ **Universal infrastructure** already exists:
  - `CallableBox::new()` works for ANY Box
  - `nyrt_callable_make()` works for ANY plugin Box

### Scalability Problem
- Adding methodRef to 100 Boxes = **600 lines** of duplicated code
- Each Box needs **identical 6-line handler** (just parameter passing)

### Solution
Add **universal pre-check** in `method_router_box::route()`:
- **15 lines** of code (one-time)
- Works for **ALL Boxes** (builtin, plugin, user)
- **97.5% reduction** for 100 Boxes (600 lines → 15 lines)

## Language Comparison

| Language | Strategy | Duplication |
|----------|----------|-------------|
| **JavaScript** | `Function.prototype.bind` | Zero |
| **Python** | Descriptor protocol | Zero |
| **Ruby** | `Object#method` inheritance | Zero |
| **Java** | Compiler `::` syntax | Zero |
| **Hakorune (Current)** | Per-Box slots | High (6 lines × N) |
| **Hakorune (Proposed)** | Universal pre-check | **Zero** |

## Implementation

### Before (Current)
```rust
// ArrayBox only
"ArrayBox" => {
    match slot {
        113 => { /* 6 lines of methodRef logic */ }
    }
}
// MapBox, StringBox, UserBoxes: NO methodRef
```

### After (Proposed)
```rust
// Add to method_router_box::route() BEFORE type dispatch
if method == "methodRef" && args.len() == 2 {
    let name = args[0].to_string();
    let arity = args[1].as_integer().unwrap_or(0);
    let recv_box = /* convert receiver to Box */;
    let cb = CallableBox::new(Some(recv_box), name, arity);
    return Ok(VMValue::from_nyash_box(Box::new(cb)));
}
// ALL Boxes now have methodRef automatically!
```

## Benefits

✅ **Zero duplication**: Single implementation for all Boxes
✅ **Perfect scalability**: 100+ Boxes with zero additional cost
✅ **Consistent behavior**: Same semantics everywhere
✅ **Simple**: 15 lines of code
✅ **Flexible**: Can still override per Box if needed

## Recommendation

**Implement universal handler NOW** — 30 minutes of work, scales forever.

## Details

See full investigation: [methodref-scalability-investigation-2025-10-10.md](methodref-scalability-investigation-2025-10-10.md)

---

**Date**: 2025-10-10
**Status**: RECOMMENDED (not yet implemented)
**Priority**: MEDIUM (current system works, but doesn't scale)
**Impact**: 97.5% code reduction for 100 Boxes
