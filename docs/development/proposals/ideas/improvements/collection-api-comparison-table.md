# 📊 Collection API Comparison Table

**Date**: 2025-10-09
**Purpose**: Side-by-side comparison of ArrayBox, MapBox, and StringBox APIs

## 🎯 Core Operations Comparison

| Operation | ArrayBox | MapBox | StringBox | Status |
|-----------|----------|--------|-----------|--------|
| **Size/Length** | `length()` → IntegerBox | `size()` → IntegerBox | ❌ Missing | 🔴 Inconsistent |
| **Get Element** | `get(index)` → Box\|NullBox ✅ | `get(key)` → StringBox(error) 😱 | ❌ N/A | 🔴 Broken |
| **Set Element** | `set(idx, val)` → StringBox("ok") | `set(key, val)` → StringBox(msg) | ❌ N/A | 🟡 Messy |
| **Has/Contains** | `contains(value)` → BoolBox | `has(key)` → BoolBox | `contains(str)` → BoolBox | ✅ Good |
| **Remove** | `remove(index)` → Box\|NullBox | `delete(key)` → StringBox(msg) | ❌ N/A | 🟡 Inconsistent |
| **Clear** | `clear()` → StringBox("ok") | `clear()` → StringBox(msg) | ❌ N/A | 🟡 Messy |
| **Is Empty** | ❌ Missing | ❌ Missing | ❌ Missing | 🟡 All missing |
| **Search** | `indexOf(val)` → IntegerBox(-1) | ❌ N/A | `find(str)` → IntegerBox(-1) | 🟡 Different names |
| **Iteration** | ❌ N/A | `forEach()` 😱 (broken) | ❌ N/A | 🔴 Non-functional |
| **JSON** | `toJSON()` → StringBox ✅ | `toJSON()` → StringBox ✅ | ❌ N/A | ✅ Good |

### Legend
- ✅ Good - Works correctly and consistently
- 🟡 Needs improvement - Works but inconsistent
- 🔴 Critical issue - Broken or major inconsistency
- ❌ N/A - Not applicable for this type
- 😱 Major problem - Critical bug or anti-pattern

---

## 🔍 Detailed Method Inventory

### ArrayBox Methods (21 total)

| Method | Signature | Return Type | Notes |
|--------|-----------|-------------|-------|
| `new()` | `() -> ArrayBox` | ArrayBox | Constructor ✅ |
| `push(item)` | `(Box) -> Box` | StringBox("ok") | 🟡 Should return null |
| `pop()` | `() -> Box` | Box \| NullBox | ✅ Correct |
| **`length()`** | `() -> IntegerBox` | IntegerBox | 🔄 Rename to `size()` |
| `get(index)` | `(IntegerBox) -> Box` | Box \| NullBox | ✅ Correct |
| `set(index, value)` | `(IntegerBox, Box) -> Box` | StringBox("ok") | 🟡 Should return null |
| `remove(index)` | `(IntegerBox) -> Box` | Box \| NullBox | ✅ Correct |
| `indexOf(value)` | `(Box) -> IntegerBox` | IntegerBox (-1 if not found) | ✅ Correct |
| `contains(value)` | `(Box) -> BoolBox` | BoolBox | ✅ Correct |
| `clear()` | `() -> Box` | StringBox("ok") | 🟡 Should return null |
| `join(delim)` | `(StringBox) -> Box` | StringBox | ✅ Correct |
| `sort()` | `() -> Box` | StringBox("ok") | 🟡 Should return null |
| `reverse()` | `() -> Box` | StringBox("ok") | 🟡 Should return null |
| `toJSON()` | `() -> Box` | StringBox | ✅ Correct |
| `slice(start, end)` | `(IntegerBox, IntegerBox) -> ArrayBox` | ArrayBox | ✅ Correct |
| ❌ `size()` | - | - | 🟢 ADD: Alias for length() |
| ❌ `isEmpty()` | - | - | 🟢 ADD: Convenience method |
| ❌ `has(index)` | - | - | 🟢 ADD: Index existence check |

---

### MapBox Methods (13 total)

| Method | Signature | Return Type | Notes |
|--------|-----------|-------------|-------|
| `new()` | `() -> MapBox` | MapBox | Constructor ✅ |
| **`get(key)`** | `(Box) -> Box` | StringBox(error) 😱 | 🔴 FIX: Return null! |
| **`set(key, value)`** | `(Box, Box) -> Box` | StringBox(msg) | 🟡 Should return null |
| `has(key)` | `(Box) -> BoolBox` | BoolBox | ✅ Correct |
| **`delete(key)`** | `(Box) -> Box` | StringBox(msg) | 🟡 Should return value\|null |
| **`clear()`** | `() -> Box` | StringBox(msg) | 🟡 Should return null |
| `keys()` | `() -> ArrayBox` | ArrayBox | ✅ Correct |
| `values()` | `() -> ArrayBox` | ArrayBox | ✅ Correct |
| `size()` | `() -> IntegerBox` | IntegerBox | ✅ Correct |
| **`forEach(callback)`** | `(Box) -> Box` | StringBox(msg) | 🔴 REMOVE: Non-functional! |
| `toJSON()` | `() -> Box` | StringBox | ✅ Correct |
| `dump()` | `() -> Box` | StringBox | ✅ Debug utility |
| `verify()` | `() -> Box` | StringBox | ✅ Debug utility |
| `stats()` | `() -> Box` | StringBox | ✅ Debug utility |
| ❌ `isEmpty()` | - | - | 🟢 ADD: Convenience method |

---

### StringBox Methods (12 total)

| Method | Signature | Return Type | Notes |
|--------|-----------|-------------|-------|
| `new(str)` | `(String) -> StringBox` | StringBox | Constructor ✅ |
| `split(delim)` | `(StringBox) -> ArrayBox` | ArrayBox | ✅ Correct |
| **`find(search)`** | `(StringBox) -> IntegerBox` | IntegerBox (-1 if not found) | 🔄 Rename to `indexOf()` |
| `replace(old, new)` | `(StringBox, StringBox) -> StringBox` | StringBox | ✅ Correct |
| `lastIndexOf(search)` | `(StringBox) -> IntegerBox` | IntegerBox (-1 if not found) | ✅ Correct |
| `trim()` | `() -> StringBox` | StringBox | ✅ Correct |
| `to_upper()` | `() -> StringBox` | StringBox | ✅ Correct |
| `to_lower()` | `() -> StringBox` | StringBox | ✅ Correct |
| `contains(search)` | `(StringBox) -> BoolBox` | BoolBox | ✅ Correct |
| `starts_with(prefix)` | `(StringBox) -> BoolBox` | BoolBox | ✅ Correct |
| `ends_with(suffix)` | `(StringBox) -> BoolBox` | BoolBox | ✅ Correct |
| `join(array)` | `(ArrayBox) -> StringBox` | StringBox | ✅ Correct |
| `to_integer()` | `() -> IntegerBox` | IntegerBox (0 on error) | ✅ Correct |
| ❌ **`size()`** | - | - | 🔴 ADD: Critical missing! |
| ❌ `isEmpty()` | - | - | 🟢 ADD: Convenience method |
| ❌ `indexOf(search)` | - | - | 🟢 ADD: Alias for find() |
| ❌ **`substring(start, end)`** | - | - | 🔴 ADD: Critical missing! |
| ❌ **`charAt(index)`** | - | - | 🔴 ADD: Critical missing! |

---

## 🌍 Comparison with Other Languages

### Size/Length Method

| Language | Array | Map/Dict | String |
|----------|-------|----------|--------|
| **JavaScript** | `.length` (property) | `.size` (property) | `.length` (property) |
| **Python** | `len(list)` (function) | `len(dict)` (function) | `len(str)` (function) |
| **Rust** | `.len()` (method) | `.len()` (method) | `.len()` (method) |
| **Hakorune (current)** | `.length()` | `.size()` | ❌ **MISSING** |
| **Hakorune (proposed)** | `.size()` ✅ | `.size()` ✅ | `.size()` ✅ |

### Get Element Behavior

| Language | Array Out-of-Bounds | Map Missing Key |
|----------|-------------------|-----------------|
| **JavaScript** | Returns `undefined` | Returns `undefined` |
| **Python** | Raises `IndexError` | Raises `KeyError` (or `dict.get()` → `None`) |
| **Rust** | `.get()` → `Option<&T>` (None) | `.get()` → `Option<&V>` (None) |
| **Hakorune (current)** | Returns `NullBox` ✅ | Returns `StringBox("Key not found")` 😱 |
| **Hakorune (proposed)** | Returns `NullBox` ✅ | Returns `NullBox` ✅ |

### Search Method Naming

| Language | Array Search | String Search |
|----------|-------------|---------------|
| **JavaScript** | `.indexOf(value)` | `.indexOf(search)` |
| **Python** | `.index(value)` | `.find(search)` |
| **Rust** | `.iter().position()` | `.find(pattern)` |
| **Hakorune (current)** | `.indexOf(value)` | `.find(search)` 🟡 |
| **Hakorune (proposed)** | `.indexOf(value)` | `.indexOf(search)` ✅ |

---

## 🎯 Proposed Unified Interface

### Core Collection Interface (All Types)

```hako
interface CollectionBox {
    // Size/emptiness
    size() -> IntegerBox           // ✅ Unified name
    isEmpty() -> BoolBox           // ✅ Convenience

    // Conversion
    toJSON() -> StringBox          // ✅ Already consistent
    toString() -> StringBox        // ✅ Already consistent (via to_string_box)
}
```

### Indexed Collection Interface (ArrayBox, StringBox)

```hako
interface IndexedCollection extends CollectionBox {
    get(index: IntegerBox) -> Box | NullBox      // ✅ Null for missing
    has(index: IntegerBox) -> BoolBox            // ✅ NEW: Index check
    indexOf(value: Box) -> IntegerBox            // ✅ Unified name (-1 for not found)
    slice(start: IntegerBox, end: IntegerBox) -> Self
}
```

### Mapped Collection Interface (MapBox)

```hako
interface MappedCollection extends CollectionBox {
    get(key: Box) -> Box | NullBox               // ✅ FIX: Return null!
    set(key: Box, value: Box) -> NullBox         // ✅ FIX: No message
    has(key: Box) -> BoolBox                     // ✅ Already correct
    delete(key: Box) -> Box | NullBox            // ✅ FIX: Return value
    keys() -> ArrayBox                           // ✅ Already correct
    values() -> ArrayBox                         // ✅ Already correct
}
```

---

## 📋 Migration Checklist

### Critical Fixes (Do First)

- [ ] **MapBox.get()**: Change return from `StringBox(error)` → `NullBox`
  - Impact: ~50-100 call sites
  - Files: `apps/selfhost/hakorune-vm/*.hako`, `apps/selfhost-compiler/*.hako`

- [ ] **StringBox.size()**: Add missing method
  - Impact: 0 (new method, no breaking change)

- [ ] **StringBox.substring()**: Add missing method
  - Impact: 0 (new method, no breaking change)

- [ ] **StringBox.charAt()**: Add missing method
  - Impact: 0 (new method, no breaking change)

### Consistency Improvements

- [ ] **ArrayBox.size()**: Add as alias for `.length()`
  - Impact: 0 (new method, no breaking change)

- [ ] **StringBox.indexOf()**: Add as alias for `.find()`
  - Impact: 0 (new method, no breaking change)

- [ ] **All.isEmpty()**: Add convenience method
  - Impact: 0 (new method, no breaking change)

### Return Type Fixes

- [ ] **Mutation methods**: Return `NullBox` instead of messages
  - `set()`, `clear()`, `push()`, `delete()`
  - Impact: Low (most code ignores return values)

### Cleanup

- [ ] **MapBox.forEach()**: Remove non-functional method
  - Impact: Low (method doesn't work anyway)

- [ ] **Deprecate**: `.length()` and `.find()`
  - Impact: 0 (keep as aliases initially)

---

## 📊 Impact Summary

| Priority | Issue | Affected Files | Difficulty | Timeline |
|----------|-------|---------------|------------|----------|
| 🔴 Critical | MapBox.get() returns error string | ~50-100 | Hard | 3-5 days |
| 🔴 Critical | StringBox missing .size() | 0 (new) | Easy | 1 day |
| 🔴 Critical | StringBox missing .substring() | 0 (new) | Easy | 1 day |
| 🔴 Critical | StringBox missing .charAt() | 0 (new) | Easy | 1 day |
| 🟡 Medium | Inconsistent size/length naming | 0 (add alias) | Easy | 1 day |
| 🟡 Medium | Return types (set/clear) | ~20-30 | Medium | 2-3 days |
| 🟢 Low | Add isEmpty() | 0 (new) | Easy | 1 day |
| 🟢 Low | Remove forEach() | ~0-5 | Easy | 1 day |

**Total Estimated Timeline**: 1-2 weeks

---

## 🔗 Related Documents

- **Full Proposal**: [unified-collection-interface-proposal.md](./unified-collection-interface-proposal.md)
- **Quick Fixes**: [collection-api-quick-fixes.md](./collection-api-quick-fixes.md)
- **Implementation Files**:
  - ArrayBox: `/home/tomoaki/git/hakorune-selfhost/src/boxes/array/mod.rs`
  - MapBox: `/home/tomoaki/git/hakorune-selfhost/src/boxes/map_box.rs`
  - StringBox: `/home/tomoaki/git/hakorune-selfhost/src/boxes/string_box.rs`

---

**Author**: Claude (Anthropic)
**Date**: 2025-10-09
**Status**: ⏳ Awaiting Review
