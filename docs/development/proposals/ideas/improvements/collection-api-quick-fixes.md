# 🚨 Collection API Quick Fixes - Priority Checklist

**Date**: 2025-10-09
**Type**: Quick Reference
**Parent**: [Unified Collection Interface Proposal](./unified-collection-interface-proposal.md)

## 🔴 Critical Issues (Fix Immediately)

### 1. MapBox.get() Returns Error String (Not Null!)

**Current Broken Behavior**:
```hako
local value = map.get("missing_key")
// Returns: null （NullBox）
```

**Clean Check (Now)**:
```hako
if value != null {
    // Key exists
}
```

**Fixed Behavior**:
```hako
local value = map.get("missing_key")
// Should return: NullBox ✅

if value != null {
    // Key exists - clean null check!
}
```

**Affected Files**: ~50-100 call sites across codebase
**Fix Priority**: 🔴 CRITICAL

---

### 2. StringBox Has No .size() Method

**Current Problem**:
```hako
local text = "Hello, World!"
// text.size() → ERROR! Method doesn't exist 😱

// Must use ugly workarounds:
local len = text.split("").length()  // Split into chars, count array
```

**Fixed Behavior**:
```hako
local text = "Hello, World!"
local len = text.size()  // → 13 ✅
```

**Fix Priority**: 🔴 CRITICAL

---

## 🟡 Consistency Issues (Fix Soon)

### 3. Inconsistent size/length Naming

| Collection | Current Method | Should Be |
|-----------|---------------|-----------|
| ArrayBox | `.length()` | `.size()` |
| MapBox | `.size()` ✅ | `.size()` |
| StringBox | ❌ (missing) | `.size()` |

**Fix**:
- Add `ArrayBox.size()` as primary method
- Keep `.length()` as deprecated alias
- Add `StringBox.size()` (new)

---

### 4. Missing Critical StringBox Methods

| Method | Purpose | Current Workaround |
|--------|---------|-------------------|
| `substring(start, end)` | Extract substring | Complex split/join logic |
| `charAt(index)` | Get character | Split into array, get element |

**Fix**: Add both methods to StringBox

---

### 5. Mutation Methods Return Useless Messages

**Current Behavior**:
```hako
map.set("key", "value")  // → null
map.clear()              // → null
array.push(item)         // → null
```

**Should Return**: `NullBox` (no message needed)

---

## 🟢 Minor Improvements (Nice to Have)

### 6. Add .isEmpty() Convenience Method

```hako
// Instead of:
if collection.size() == 0 { ... }

// Provide:
if collection.isEmpty() { ... }
```

**Add to**: ArrayBox, MapBox, StringBox

---

### 7. Remove Non-Functional MapBox.forEach()

**Issue**: Method exists but callback is never executed (line 255-259 in map_box.rs)

```hako
map.forEach(callback)  // Callback is ignored! 😱
// Returns: StringBox("Iterated over 5 items") (lie!)
```

**Fix**: Remove method entirely

---

## 📋 Quick Implementation Checklist

### Phase 1: Add New Methods (No Breaking Changes)
- [ ] Add `StringBox.size()` → IntegerBox
- [ ] Add `StringBox.substring(start, end)` → StringBox
- [ ] Add `StringBox.charAt(index)` → StringBox | NullBox
- [ ] Add `StringBox.indexOf()` (alias for `.find()`)
- [ ] Add `ArrayBox.size()` (alias for `.length()`)
- [ ] Add `ArrayBox.isEmpty()` → BoolBox
- [ ] Add `MapBox.isEmpty()` → BoolBox
- [ ] Add `StringBox.isEmpty()` → BoolBox

### Phase 2: Fix Critical Bug (Breaking Change)
- [ ] Change `MapBox.get(key)` to return `NullBox` (not error string)
- [ ] Add feature flag: `HAKO_COLLECTION_V2=1`
- [ ] Migrate ~50-100 call sites
- [ ] Remove string-based error detection

### Phase 3: Fix Return Types (Semi-Breaking)
- [ ] Change mutation methods to return `NullBox`:
  - `map.set()`, `map.clear()`, `map.delete()`
  - `array.push()`, `array.set()`, `array.clear()`

### Phase 4: Deprecation & Cleanup
- [ ] Add deprecation warnings for `.length()` and `.find()`
- [ ] Remove `MapBox.forEach()`
- [ ] Update documentation

---

## 🔍 Migration Script Template

### Find All MapBox.get() Error Checks

```bash
#!/bin/bash
# Find files with MapBox.get() error string checks

echo "=== Files with 'Key not found' checks ==="
grep -r "Key not found" apps/ --include="*.hako" --include="*.nyash" -l

echo ""
echo "=== Suggested replacements ==="
grep -r "RegexFlow.find_from.*Key not found" apps/ --include="*.hako" --include="*.nyash" -n
```

### Auto-Migration (Use with Caution!)

```bash
#!/bin/bash
# Migrate MapBox.get() error checks to null checks

find apps/ -name "*.hako" -o -name "*.nyash" | while read file; do
    # Pattern 1: RegexFlow.find_from check
    sed -i 's/RegexFlow\.find_from(\([^,]*\), "Key not found:", 0) < 0/\1 != null/g' "$file"

    # Pattern 2: startsWith check
    sed -i 's/\([^.]*\)\.toString()\.startsWith("Key not found")/\1 == null/g' "$file"
done

echo "Migration complete. Review changes with 'git diff'"
```

---

## 📊 Impact Summary

| Change | Files Affected | Difficulty | Timeline |
|--------|---------------|------------|----------|
| Add StringBox.size() | 0 (new method) | Easy | 1 day |
| Fix MapBox.get() | ~50-100 | Hard | 3-5 days |
| Add isEmpty() | 0 (new method) | Easy | 1 day |
| Fix return types | ~20-30 | Medium | 2-3 days |
| Add substring/charAt | 0 (new method) | Easy | 1-2 days |

**Total Timeline**: 1-2 weeks

---

## 🎯 Recommended Priority Order

1. **Week 1**: Add non-breaking methods (size, isEmpty, substring, charAt)
2. **Week 2**: Fix MapBox.get() critical bug with feature flag
3. **Week 3**: Fix return types, add deprecation warnings
4. **Later**: Remove deprecated methods after 1-2 releases

---

## 📚 Related Files

- **Full Proposal**: [unified-collection-interface-proposal.md](./unified-collection-interface-proposal.md)
- **Implementation Files**:
  - `/home/tomoaki/git/hakorune-selfhost/src/boxes/array/mod.rs`
  - `/home/tomoaki/git/hakorune-selfhost/src/boxes/map_box.rs`
  - `/home/tomoaki/git/hakorune-selfhost/src/boxes/string_box.rs`

---

**Status**: ⏳ Awaiting implementation
**Author**: Claude (Anthropic)
