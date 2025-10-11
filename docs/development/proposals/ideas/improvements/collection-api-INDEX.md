# 📦 Collection API Unification - Complete Documentation Index

**Date**: 2025-10-09
**Status**: Proposal Phase
**Author**: Claude (Anthropic)

---

## 🎯 Overview

This proposal addresses critical inconsistencies in Hakorune's collection Box APIs (ArrayBox, MapBox, StringBox). The main issues are:

1. **MapBox.get()** returns error strings instead of null (major bug)
2. **StringBox** is missing critical methods (size, substring, charAt)
3. **Inconsistent naming** across collection types (length vs size)
4. **Messy return values** from mutation methods

**Goal**: Create a unified, predictable, "箱らしい" (Box-like) collection interface.

---

## 📚 Documentation Structure

### 1. 📋 [Quick Fixes Checklist](./collection-api-quick-fixes.md)
**Start here for implementation!**

- 🔴 Critical issues that need immediate fixes
- 🟡 Consistency improvements
- 🟢 Minor enhancements
- Priority-ordered implementation checklist
- Migration scripts

**Best for**: Developers ready to implement fixes

---

### 2. 📊 [API Comparison Table](./collection-api-comparison-table.md)
**Side-by-side comparison of all methods**

- Complete method inventory (21 ArrayBox, 13 MapBox, 12 StringBox)
- Comparison with JavaScript, Python, Rust
- Status indicators (✅ Good, 🟡 Needs improvement, 🔴 Critical)
- Proposed unified interface
- Migration impact summary

**Best for**: Understanding current state and proposed changes

---

### 3. 🔄 [Before & After Examples](./collection-api-before-after.md)
**Visual guide with real code examples**

- Problem 1: MapBox.get() error string hell
- Problem 2: StringBox missing critical methods
- Problem 3: Inconsistent naming (size vs length)
- Problem 4: Search method naming
- Problem 5: Mutation return values
- Migration path visualization

**Best for**: Understanding the "why" behind changes

---

### 4. 📖 [Full Proposal Document](./unified-collection-interface-proposal.md)
**Complete technical specification (12,000+ words)**

- Detailed API inventory and analysis
- Unified interface design
- Language comparison (JS, Python, Rust)
- Specific recommendations for each Box
- Complete migration plan (5 phases)
- Impact analysis and timeline
- Code examples and edge cases
- Open questions and decisions

**Best for**: Deep technical review and decision-making

---

## 🚨 Critical Issues Summary

### Issue #1: MapBox.get() Returns Error String (NOT NULL!)

**Current Behavior** (BROKEN):
```hako
local value = map.get("missing_key")
// Returns: StringBox("Key not found: missing_key") 😱

// Forces ugly string-based error detection:
if RegexFlow.find_from(value, "Key not found:", 0) >= 0 {
    // Handle missing key
}
```

**Fixed Behavior**:
```hako
local value = map.get("missing_key")
// Returns: NullBox ✅

// Clean null check:
if value == null {
    // Handle missing key
}
```

**Impact**: ~50-100 call sites across codebase
**Priority**: 🔴 CRITICAL

---

### Issue #2: StringBox Missing Critical Methods

**Missing Methods**:
- ❌ `.size()` - No way to get string length!
- ❌ `.substring(start, end)` - No substring extraction!
- ❌ `.charAt(index)` - No character access!

**Current Workarounds** (ugly):
```hako
// Get length: must split into chars and count
local len = text.split("").length()

// Get substring: complex manual loop
local chars = text.split("")
local result = ""
loop(i < n) { result = result + chars.get(i); i = i + 1 }

// Get character: split and extract
local char = text.split("").get(0)
```

**Impact**: Affects all string processing code
**Priority**: 🔴 CRITICAL

---

### Issue #3: Inconsistent Naming

| Collection | Current Method | Should Be |
|-----------|---------------|-----------|
| ArrayBox | `.length()` | `.size()` |
| MapBox | `.size()` ✅ | `.size()` |
| StringBox | ❌ (missing) | `.size()` |

**Impact**: Confusing for users, cognitive overhead
**Priority**: 🟡 HIGH

---

## 🎯 Recommended Implementation Order

### Week 1: Non-Breaking Additions
1. ✅ Add `StringBox.size()` - 1 day
2. ✅ Add `StringBox.substring()` - 1 day
3. ✅ Add `StringBox.charAt()` - 1 day
4. ✅ Add `ArrayBox.size()` (alias) - 1 day
5. ✅ Add `isEmpty()` to all - 1 day

### Week 2: Critical Bug Fix
6. 🔴 Fix `MapBox.get()` to return null - 3-5 days
   - Add feature flag `HAKO_COLLECTION_V2=1`
   - Migrate ~50-100 call sites
   - Use migration scripts

### Week 3: Polish & Cleanup
7. 🟡 Fix mutation return types - 2-3 days
8. 🟢 Add deprecation warnings - 1 day
9. 🟢 Remove `MapBox.forEach()` - 1 day

---

## 📊 Files Affected (Estimated)

| Change | Files | Difficulty | Timeline |
|--------|-------|------------|----------|
| Add StringBox methods | 0 (new) | Easy | 3 days |
| Fix MapBox.get() | ~50-100 | Hard | 3-5 days |
| Add size/isEmpty | 0 (new) | Easy | 2 days |
| Fix return types | ~20-30 | Medium | 2-3 days |

**Total**: 1-2 weeks development + 1-2 months migration period

---

## 🔧 Implementation Files

### Source Files to Modify
- `/home/tomoaki/git/hakorune-selfhost/src/boxes/array/mod.rs` - ArrayBox
- `/home/tomoaki/git/hakorune-selfhost/src/boxes/map_box.rs` - MapBox
- `/home/tomoaki/git/hakorune-selfhost/src/boxes/string_box.rs` - StringBox

### Call Sites to Migrate (Examples)
- `/home/tomoaki/git/hakorune-selfhost/selfhost/hakorune-vm/load_handler.hako:31`
- `/home/tomoaki/git/hakorune-selfhost/selfhost/hakorune-vm/terminator_handler.hako:87,142`
- `/home/tomoaki/git/hakorune-selfhost/selfhost/hakorune-vm/value_manager.hako:13`
- `/home/tomoaki/git/hakorune-selfhost/selfhost/hakorune-vm/phi_handler.hako:113`
- `/home/tomoaki/git/hakorune-selfhost/selfhost/compiler/pipeline_v2/using_resolver_box.hako:125`

---

## 🛠️ Migration Tools

### Find Affected Files
```bash
# Find MapBox.get() error checks
grep -r "Key not found" apps/ --include="*.hako" --include="*.nyash" -l

# Count total occurrences
grep -r "\.get(" apps/ --include="*.hako" --include="*.nyash" | wc -l
```

### Auto-Migration Script
```bash
#!/bin/bash
# Migrate MapBox.get() error checks to null checks

find apps/ -name "*.hako" -o -name "*.nyash" | while read file; do
    # Pattern 1: RegexFlow.find_from check
    sed -i 's/RegexFlow\.find_from(\([^,]*\), "Key not found:", 0) < 0/\1 != null/g' "$file"

    # Pattern 2: startsWith check
    sed -i 's/\([^.]*\)\.toString()\.startsWith("Key not found")/\1 == null/g' "$file"
done

echo "Migration complete. Review with 'git diff'"
```

---

## 🎯 Success Criteria

### Phase 1 Complete When:
- [ ] All new methods implemented (size, isEmpty, substring, charAt, indexOf)
- [ ] All tests pass
- [ ] No breaking changes
- [ ] Documentation updated

### Phase 2 Complete When:
- [ ] MapBox.get() returns NullBox
- [ ] All call sites migrated
- [ ] No string-based error detection remains
- [ ] Feature flag functional

### Final Success When:
- [ ] Zero API inconsistencies across collections
- [ ] All critical methods present (size, substring, charAt)
- [ ] Clean null-based error handling everywhere
- [ ] "箱らしい" unified design achieved

---

## 📖 Quick Reference

### Current API (Inconsistent)
```hako
// Size/Length - INCONSISTENT! 😱
array.length()    // ArrayBox
map.size()        // MapBox
// string.???    // StringBox - MISSING!

// Get element - BROKEN! 😱
array.get(0)           // → Box | NullBox ✅
map.get("key")         // → StringBox(error) 😱
// string.get(0)?      // → N/A

// Search - INCONSISTENT! 😱
array.indexOf(x)       // ArrayBox
string.find(x)         // StringBox - Different name!
```

### Proposed API (Unified)
```hako
// Size/Length - UNIFIED! ✅
array.size()      // All collections
map.size()
string.size()

// Get element - FIXED! ✅
array.get(0)           // → Box | NullBox
map.get("key")         // → Box | NullBox (FIXED!)
string.charAt(0)       // → StringBox | NullBox (NEW!)

// Search - UNIFIED! ✅
array.indexOf(x)       // All use indexOf
string.indexOf(x)      // (find() kept as alias)

// BONUS: New convenience methods ✅
array.isEmpty()        // All collections
map.isEmpty()
string.isEmpty()

string.substring(0, 5) // NEW!
```

---

## 🔗 Related Resources

### Internal Documentation
- [Box System Reference](/docs/reference/boxes-system/box-reference.md)
- [Everything is Box Philosophy](/docs/reference/boxes-system/everything-is-box.md)
- [Language Quick Reference](/docs/reference/language/quick-reference.md)

### External Inspirations
- **JavaScript**: Map.get() returns undefined (not error!)
- **Python**: dict.get() returns None (not error!)
- **Rust**: HashMap.get() returns Option<&V> (type-safe!)

---

## 💬 Discussion & Feedback

### Open Questions
1. Should `delete()` return deleted value or boolean?
   - **Recommendation**: Return value (like Rust)
2. Support negative indices in slice/substring?
   - **Recommendation**: Yes (like Python/JS)
3. Keep deprecated methods as aliases?
   - **Recommendation**: Yes, for 1-2 releases

### Next Steps
1. ✅ Review this proposal
2. ⏳ Decide on priority order
3. ⏳ Create implementation branch
4. ⏳ Begin Phase 1 (non-breaking additions)

---

## 📝 Version History

- **2025-10-09**: Initial proposal created
  - Full specification documented
  - Migration plan defined
  - Impact analysis completed
  - Code examples provided

---

**Status**: ⏳ Awaiting Review & Implementation
**Maintainer**: Hakorune Core Team
**Reviewer**: TBD

---

## 🎉 TL;DR

**3 Critical Problems**:
1. MapBox.get() returns error string (not null) 🔴
2. StringBox missing size/substring/charAt 🔴
3. Inconsistent naming (length vs size) 🟡

**Solution**:
1. Fix MapBox.get() → return null
2. Add StringBox methods (size, substring, charAt)
3. Unify naming (all use .size())

**Timeline**: 1-2 weeks
**Impact**: ~50-100 files
**Result**: Beautiful, consistent, "箱らしい" collection API ✨
