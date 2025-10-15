# 📦 Unified Collection Interface Proposal

**Date**: 2025-10-09
**Status**: Proposal
**Type**: API Consistency & Design Improvement

## 🎯 Executive Summary

**Problem**: ArrayBox, MapBox, and StringBox have inconsistent APIs, particularly in error handling:
- **MapBox.get()** returns `StringBox("Key not found: ...")` error message
- **ArrayBox.get()** returns `NullBox` for missing index
- **StringBox** has different method naming (`.find()` vs `.indexOf()`)

**Solution**: Create unified "Collection Box Interface" with consistent:
- Error handling (null returns vs error messages)
- Method naming (size/length/len)
- Core operations (get/set/has/clear)

**Impact**:
- **Breaking Change**: Yes (MapBox.get() behavior change)
- **Affected Code**: ~50-100 call sites (estimated)
- **Benefit**: Safer, predictable, "箱らしい" API

> Update (Phase 15.7)
- StringBox: builder lowering extended — `lastIndexOf` and `replace` added; `find` is an alias of `indexOf`.
- Externs registry includes: `nyrt.string.{length,indexOf,lastIndexOf,substring,charAt,replace}`. VM implements them via core crate.
- See also: docs/mir/externs/README.md for the canonical Extern list and lowering policy.

---

## 📊 1. Current API Comparison

### Complete Method Inventory

| Operation | ArrayBox | MapBox | StringBox |
|-----------|----------|--------|-----------|
| **Size/Length** | `length()` → IntegerBox | `size()` → IntegerBox | N/A (no size method) |
| **Get Element** | `get(index)` → Box \| NullBox | `get(key)` → Box \| StringBox(error) | N/A |
| **Set Element** | `set(index, value)` → NullBox | `set(key, value)` → NullBox | N/A (immutable) |
| **Has/Contains** | `contains(value)` → BoolBox | `has(key)` → BoolBox | `contains(search)` → BoolBox |
| **Remove** | `remove(index)` → Box \| NullBox | `delete(key)` → StringBox(msg) | N/A (immutable) |
| **Clear** | `clear()` → NullBox | `clear()` → NullBox | N/A |
| **Search** | `indexOf(value)` → IntegerBox(-1) | N/A | `find(search)` → IntegerBox(-1) |
| **Keys/Indices** | N/A | `keys()` → ArrayBox | N/A |
| **Values** | N/A | `values()` → ArrayBox | N/A |
| **Iteration** | N/A | `forEach(callback)` → StringBox(msg) | N/A |
| **Conversion** | `toJSON()` → StringBox | `toJSON()` → StringBox | N/A |
| **Slice/Substring** | `slice(start, end)` → ArrayBox | N/A | N/A (no substring method!) |
| **Join/Split** | `join(delim)` → StringBox | N/A | `split(delim)` → ArrayBox |
| **Sort/Reverse** | `sort()`, `reverse()` | N/A | N/A |
| **Debug** | N/A | `dump()`, `verify()`, `stats()` | N/A |

### 🚨 Critical Inconsistencies

#### 1️⃣ **Error Handling Nightmare**
```nyash
// ArrayBox: Returns NullBox (type-safe)
local value = array.get(999)  // → NullBox (no error)

// MapBox: Returns error STRING (breaks code!)
local value = map.get("missing")  // → StringBox("Key not found: missing")

// Real-world pain:
if value.startsWith("Key not found") {  // 😱 String check for errors!
    // Handle missing key
}
```

**Evidence from codebase** (`selfhost/hakorune-vm/load_handler.hako:31`):
```nyash
// Fix: MapBox.get() returns StringBox("Key not found") instead of null
```

#### 2️⃣ **Method Naming Chaos**
```nyash
// ArrayBox uses "length"
array.length()  // ✅

// MapBox uses "size"
map.size()      // ✅

// StringBox has NO size/length method!
// Must use .split("").length() workaround

// Search methods:
array.indexOf(value)    // ✅
string.find(search)     // 🤔 Different name!
```

#### 3️⃣ **Missing Critical Methods**
```nyash
// StringBox: No substring() method!
// Users must use workarounds

// MapBox: No indexOf() for values
// Can't find which key has a value

// ArrayBox: No slice() with negative indices
// Limited compared to JS/Python
```

---

## 🏗️ 2. Unified Collection Interface Design

### Core Principle: "箱らしい統一性"

**Philosophy**: All collections should behave predictably:
- **Null for missing** (not error strings)
- **Consistent naming** (size/get/has/clear)
- **Fail-fast** (runtime errors for invalid operations)
- **Explicit over implicit**

### 2.1 Conceptual Interface

```hako
// Conceptual interface (not actual Hakorune syntax)
interface CollectionBox {
    // ===== Core Operations (ALL collections) =====

    // Size/Length - CONSISTENT naming
    size() -> IntegerBox                 // Unified: "size" for all
    isEmpty() -> BoolBox                 // NEW: Convenience method

    // Clear/Reset
    clear() -> NullBox                   // CHANGED: Return null (no message)

    // Conversion
    toJSON() -> StringBox                // Already consistent ✅
    toString() -> StringBox              // Already via to_string_box ✅

    // ===== Type-specific adaptations =====
    // Implemented differently per collection type
}

interface IndexedCollection extends CollectionBox {
    // ArrayBox, StringBox (indexed access)
    get(index: IntegerBox) -> Box | NullBox    // Null for out-of-bounds
    has(index: IntegerBox) -> BoolBox          // NEW: Index existence
    indexOf(value: Box) -> IntegerBox          // -1 for not found
    slice(start: IntegerBox, end: IntegerBox) -> Self
}

interface MappedCollection extends CollectionBox {
    // MapBox (key-value access)
    get(key: Box) -> Box | NullBox       // CHANGED: Return null, not error!
    set(key: Box, value: Box) -> NullBox // CHANGED: Return null, not message
    has(key: Box) -> BoolBox             // Already consistent ✅
    delete(key: Box) -> Box | NullBox    // CHANGED: Return deleted value or null
    keys() -> ArrayBox                   // Already consistent ✅
    values() -> ArrayBox                 // Already consistent ✅
}
```

### 2.2 Unified Method Signatures

#### ✅ **Keep As-Is** (Already Consistent)

| Method | All Collections | Return Type | Notes |
|--------|----------------|-------------|-------|
| `toJSON()` | ArrayBox, MapBox | StringBox | JSON serialization ✅ |
| `has(key)` | MapBox | BoolBox | Existence check ✅ |
| `contains(value)` | ArrayBox, StringBox | BoolBox | Value search ✅ |

#### 🔄 **Rename for Consistency**

| Current | Unified | Collections | Reason |
|---------|---------|------------|--------|
| `length()` | `size()` | ArrayBox | Match MapBox naming |
| `find()` | `indexOf()` | StringBox | Match ArrayBox naming |
| N/A | `size()` | StringBox | Add missing method |

#### ➕ **Add for Completeness**

| Method | Add To | Signature | Purpose |
|--------|--------|-----------|---------|
| `isEmpty()` | All | `() -> BoolBox` | Convenience: `size() == 0` |
| `has(index)` | ArrayBox | `(IntegerBox) -> BoolBox` | Index validity check |
| `substring()` | StringBox | `(start, end) -> StringBox` | Extract substring |
| `charAt()` | StringBox | `(IntegerBox) -> StringBox \| NullBox` | Get character at index |

#### 🗑️ **Remove/Deprecate**

| Method | Collection | Reason |
|--------|-----------|--------|
| `forEach()` | MapBox | Non-functional (callback not executed) |
| Return messages | All | Use NullBox instead of StringBox("ok") |

#### 🔧 **Fix Critical Bugs**

| Method | Collection | Current Behavior | Fixed Behavior |
|--------|-----------|-----------------|----------------|
| `get(key)` | MapBox | NullBox | **NullBox** |
| `delete(key)` | MapBox | NullBox | **NullBox** |
| `set(key, value)` | MapBox | StringBox("Set key: X") | **NullBox** |
| `clear()` | All | StringBox("ok"/"Map cleared") | **NullBox** |

---

## 📋 3. Detailed Recommendations by Box

### 3.1 ArrayBox Changes

#### ✅ **Keep** (9 methods)
- `push(item)` - Add element (but return NullBox instead of "ok")
- `pop()` - Remove last (already returns Box|NullBox ✅)
- `get(index)` - Get element (already returns Box|NullBox ✅)
- `set(index, value)` - Set element (but return NullBox)
- `remove(index)` - Remove element (already returns Box|NullBox ✅)
- `indexOf(value)` - Search (already returns IntegerBox(-1) ✅)
- `contains(value)` - Existence check (already returns BoolBox ✅)
- `toJSON()` - Serialization ✅
- `slice(start, end)` - Extract subarray ✅

#### 🔄 **Rename** (1 method)
- `length()` → `size()` - Unify with MapBox
- **Keep `length()` as alias** for backward compatibility (deprecate later)

#### ➕ **Add** (2 methods)
```hako
// Check if index is valid (prevent out-of-bounds)
has(index: IntegerBox) -> BoolBox {
    local len = me.items.size()
    return index >= 0 && index < len
}

// Convenience: check if empty
isEmpty() -> BoolBox {
    return me.size() == 0
}
```

#### 🔧 **Fix Return Types** (4 methods)
- `push(item)` → Return **NullBox**
- `set(index, value)` → Return **NullBox**
- `clear()` → Return **NullBox**
- `sort()`, `reverse()` → Return **NullBox**

#### 🗑️ **Remove** (0 methods)
- N/A - All ArrayBox methods are useful

---

### 3.2 MapBox Changes

#### ✅ **Keep** (6 methods)
- `has(key)` - Existence check ✅
- `keys()` - Get all keys ✅
- `values()` - Get all values ✅
- `toJSON()` - Serialization ✅
- `dump()`, `verify()`, `stats()` - Debug utilities ✅

#### 🔧 **Fix Critical - Return NullBox** (4 methods)
```hako
// BEFORE (broken):
get(key) -> StringBox("Key not found: X")  // 😱
set(key, value) -> StringBox("Set key: X")
delete(key) -> StringBox("Deleted key: X")
clear() -> StringBox("Map cleared")

// AFTER (fixed):
get(key) -> Box | NullBox              // ✅ Null for missing
set(key, value) -> NullBox             // ✅ Consistent with other mutations
delete(key) -> Box | NullBox           // ✅ Return deleted value
clear() -> NullBox                     // ✅ No message needed
```

#### ➕ **Add** (1 method)
```hako
// Convenience: check if empty
isEmpty() -> BoolBox {
    return me.size() == 0
}
```

#### 🗑️ **Remove** (1 method)
- `forEach(callback)` - **Remove**: Non-functional (callback never executed, line 255-259)

#### 🤔 **Rename Consideration**
- `delete(key)` → `remove(key)` - Match ArrayBox naming?
- **Decision**: Keep `delete()` (matches JavaScript Map.delete())

---

### 3.3 StringBox Changes

#### ✅ **Keep** (12 methods)
- `split(delim)` → ArrayBox ✅
- `trim()` → StringBox ✅
- `to_upper()` → StringBox ✅
- `to_lower()` → StringBox ✅
- `contains(search)` → BoolBox ✅
- `starts_with(prefix)` → BoolBox ✅
- `ends_with(suffix)` → BoolBox ✅
- `join(array)` → StringBox ✅
- `replace(old, new)` → StringBox ✅
- `to_integer()` → IntegerBox ✅
- `lastIndexOf(search)` → IntegerBox ✅

#### 🔄 **Rename** (1 method)
- `find(search)` → `indexOf(search)` - Match ArrayBox naming
- **Keep `find()` as alias** for backward compatibility

#### ➕ **Add** (4 methods)
```hako
// String length (critical missing method!)
size() -> IntegerBox {
    return me.value.chars().count()  // UTF-8 aware
}

// Check if empty
isEmpty() -> BoolBox {
    return me.size() == 0
}

// Get character at index (like ArrayBox.get)
charAt(index: IntegerBox) -> StringBox | NullBox {
    local chars = me.value.chars()
    if index < 0 || index >= chars.count() {
        return null
    }
    return chars.nth(index)
}

// Extract substring (like ArrayBox.slice)
substring(start: IntegerBox, end: IntegerBox) -> StringBox {
    local chars = me.value.chars()
    local len = chars.count()

    // Normalize indices
    if start < 0 { start = 0 }
    if end < 0 || end > len { end = len }

    // Extract substring
    return chars.skip(start).take(end - start).collect()
}
```

#### 🗑️ **Remove** (0 methods)
- N/A - All StringBox methods are useful

---

## 🌍 4. Comparison with Other Languages

### JavaScript (Reference Model)

```javascript
// Array
arr.length        // ✅ Property (not method)
arr[0]            // ✅ Returns undefined (not null)
arr.indexOf(x)    // ✅ Returns -1 if not found
arr.slice(0, 2)   // ✅ Extract subarray

// Map
map.size          // ✅ Property (not method)
map.get(key)      // ✅ Returns undefined (not error!)
map.set(key, val) // ✅ Returns map (chainable)
map.has(key)      // ✅ Boolean check
map.delete(key)   // ✅ Returns boolean

// String
str.length        // ✅ Property
str.indexOf(x)    // ✅ Returns -1 if not found
str.substring(0,2)// ✅ Extract substring
str.charAt(0)     // ✅ Get character
```

### Python (Reference Model)

```python
# list
len(arr)          # ✅ Global function
arr[0]            # ✅ Raises IndexError (fail-fast!)
arr.index(x)      # ✅ Raises ValueError if not found
arr[0:2]          # ✅ Slice syntax

# dict
len(d)            # ✅ Global function
d.get(key)        # ✅ Returns None (not error!)
d[key] = val      # ✅ Assignment syntax
key in d          # ✅ Existence check
d.pop(key)        # ✅ Remove and return value

# str
len(s)            # ✅ Global function
s.find(x)         # ✅ Returns -1 if not found
s[0:2]            # ✅ Slice syntax
s[0]              # ✅ Single character
```

### Rust (Type-Safety Model)

```rust
// Vec
vec.len()         // ✅ Method (usize)
vec.get(0)        // ✅ Returns Option<&T>
vec.push(x)       // ✅ Returns () (unit)
vec.clear()       // ✅ Returns () (unit)

// HashMap
map.len()         // ✅ Method (usize)
map.get(key)      // ✅ Returns Option<&V>
map.insert(k, v)  // ✅ Returns Option<V> (old value)
map.remove(key)   // ✅ Returns Option<V>
map.contains_key()// ✅ Boolean check

// String
s.len()           // ✅ Byte length
s.chars().count() // ✅ Character count (UTF-8)
s.find(pat)       // ✅ Returns Option<usize>
s.get(0..2)       // ✅ Returns Option<&str>
```

### 🎯 **Key Takeaways**

| Pattern | JS | Python | Rust | Hakorune Should |
|---------|----|----|------|-----------------|
| **Missing value** | `undefined` | `None` | `Option<T>` | **NullBox** ✅ |
| **Size method** | `.length` / `.size` | `len()` | `.len()` | **`.size()`** |
| **Get element** | `[index]` (undefined) | `[index]` (error) | `.get()` (Option) | **`.get()` → null** ✅ |
| **Search** | `.indexOf()` | `.index()` / `.find()` | `.find()` | **`.indexOf()`** |
| **Mutation returns** | `self` / `undefined` | `None` | `()` / `Option<T>` | **NullBox** |

**Recommendation**: Follow **Rust's Option<T> pattern** using NullBox (type-safe, explicit, "箱らしい")

---

## 🚧 5. Migration Plan

### Phase 1: Add New Methods (No Breaking Changes)

**Timeline**: 1-2 days

```hako
// ArrayBox
+ size() -> IntegerBox              // Alias for length()
+ isEmpty() -> BoolBox
+ has(index) -> BoolBox

// MapBox
+ isEmpty() -> BoolBox

// StringBox
+ size() -> IntegerBox              // NEW: Critical missing method!
+ isEmpty() -> BoolBox
+ indexOf(search) -> IntegerBox     // Alias for find()
+ charAt(index) -> StringBox|null
+ substring(start, end) -> StringBox
```

**Implementation**:
1. Add methods to `src/boxes/array/mod.rs`
2. Add methods to `src/boxes/map_box.rs`
3. Add methods to `src/boxes/string_box.rs`
4. Add tests for new methods
5. Update documentation

**Impact**: **Zero** (only additions, no changes)

---

### Phase 2: Add Deprecation Warnings

**Timeline**: 1 day

```rust
// ArrayBox.length() - Add deprecation warning
pub fn length(&self) -> Box<dyn NyashBox> {
    if std::env::var("HAKO_WARN_DEPRECATED").is_ok() {
        eprintln!("Warning: ArrayBox.length() is deprecated, use .size()");
    }
    self.size()  // Delegate to new method
}

// StringBox.find() - Add deprecation warning
pub fn find(&self, search: &str) -> Box<dyn NyashBox> {
    if std::env::var("HAKO_WARN_DEPRECATED").is_ok() {
        eprintln!("Warning: StringBox.find() is deprecated, use .indexOf()");
    }
    self.indexOf(search)  // Delegate to new method
}
```

**Impact**: **Zero** (warnings only, code still works)

---

### Phase 3: Fix Return Types (Semi-Breaking)

**Timeline**: 2-3 days

#### 3.1 Change Return Values to NullBox

```hako
// BEFORE:
map.set("key", "value")  // → StringBox("Set key: key")
map.clear()              // → StringBox("Map cleared")
array.push(item)         // → null

// AFTER:
map.set("key", "value")  // → NullBox
map.clear()              // → NullBox
array.push(item)         // → NullBox
```

**Migration Strategy**:
1. Add environment variable gate: `HAKO_COLLECTION_V2=1`
2. If set, use new behavior (return NullBox)
3. If not set, use old behavior (return StringBox)
4. Document migration path

```rust
// Example gating (MapBox.set):
pub fn set(&self, key: Box<dyn NyashBox>, value: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
    let key_str = key.to_string_box().value;
    self.data.write().unwrap().insert(key_str.clone(), value);

    if std::env::var("HAKO_COLLECTION_V2").is_ok() {
        Box::new(NullBox::new())  // New behavior
    } else {
        Box::new(StringBox::new(&format!("Set key: {}", key_str)))  // Old
    }
}
```

**Impact**: **Low** (most code ignores return values from set/clear)

---

### Phase 4: Fix MapBox.get() Critical Bug (BREAKING)

**Timeline**: 3-5 days (most critical!)

#### Current Bug (Major Pain Point)

```hako
// BROKEN: Returns error STRING
local value = map.get("missing_key")
// value = StringBox("Key not found: missing_key")

// Forces ugly workarounds:
if value.toString().startsWith("Key not found") {
    // Handle missing key - THIS IS TERRIBLE! 😱
}
```

**Evidence from codebase** (~50-100 call sites):
- `selfhost/hakorune-vm/load_handler.hako:31`
- `selfhost/hakorune-vm/terminator_handler.hako:87, 142`
- `selfhost/hakorune-vm/value_manager.hako:13`
- `selfhost/hakorune-vm/phi_handler.hako:113`
- `selfhost/compiler/pipeline_v2/using_resolver_box.hako:125`

#### Fixed Behavior

```hako
// FIXED: Returns NullBox
local value = map.get("missing_key")
// value = NullBox

// Clean null check:
if value == null {
    // Handle missing key - BEAUTIFUL! ✅
}
```

#### Migration Strategy

**Step 1: Search All MapBox.get() Call Sites**
```bash
grep -r "\.get(" apps/ --include="*.hako" --include="*.nyash" | wc -l
# Estimated: 50-100 call sites
```

**Step 2: Add Temporary Helper Method**
```hako
// Add to MapBox (temporary):
getSafe(key) -> Box | NullBox {
    // New behavior (return null for missing)
    local key_str = key.toString()
    if me.has(key) {
        return me.data.get(key_str).clone()
    }
    return null
}
```

**Step 3: Migrate Call Sites**
```hako
// BEFORE:
local value = map.get("key")
if RegexFlow.find_from(value, "Key not found:", 0) < 0 {
    // Key exists - use value
}

// AFTER:
local value = map.get("key")  // Now returns null!
if value != null {
    // Key exists - use value
}
```

**Step 4: Enable via Feature Flag**
```rust
pub fn get(&self, key: Box<dyn NyashBox>) -> Box<dyn NyashBox> {
    let key_str = key.to_string_box().value;
    let guard = self.data.read().unwrap();

    match guard.get(&key_str) {
        Some(value) => value.clone_box(),
        None => {
            if std::env::var("HAKO_COLLECTION_V2").is_ok() {
                Box::new(NullBox::new())  // ✅ Fixed!
            } else {
                Box::new(StringBox::new(&format!("Key not found: {}", key_str)))  // Legacy
            }
        }
    }
}
```

**Step 5: Batch Update with Script**
```bash
# Create migration script
cat > migrate_map_get.sh << 'EOF'
#!/bin/bash
# Find and update MapBox.get() error checks

find apps/ -name "*.hako" -o -name "*.nyash" | while read file; do
    # Pattern 1: "Key not found" string check
    sed -i 's/RegexFlow\.find_from(\([^,]*\), "Key not found:", 0) < 0/\1 != null/g' "$file"

    # Pattern 2: startsWith check
    sed -i 's/\([^.]*\)\.toString()\.startsWith("Key not found")/\1 == null/g' "$file"
done
EOF

chmod +x migrate_map_get.sh
```

**Impact**: **HIGH** (50-100 call sites, but scriptable migration)

---

### Phase 5: Remove Deprecated Methods

**Timeline**: 1 day (after 1-2 release cycles)

```rust
// Remove these methods entirely:
// - ArrayBox.length() (use .size())
// - StringBox.find() (use .indexOf())
// - MapBox.forEach() (non-functional)
```

**Impact**: **Medium** (only affects code not migrated in Phase 2)

---

## 📊 6. Migration Impact Summary

### Estimated Call Sites Affected

| Change | Affected Methods | Est. Call Sites | Difficulty |
|--------|-----------------|-----------------|------------|
| **Phase 1: Add methods** | 10 new methods | 0 (additions only) | Easy ✅ |
| **Phase 2: Deprecation** | 2 methods | 0 (warnings only) | Easy ✅ |
| **Phase 3: Return types** | 6 methods | ~20-30 | Medium 🟡 |
| **Phase 4: MapBox.get()** | 1 method | **~50-100** | Hard 🔴 |
| **Phase 5: Remove deprecated** | 3 methods | ~10-20 | Easy ✅ |

### Total Estimated Impact
- **Total files to modify**: ~60-120
- **Total timeline**: 1-2 weeks
- **Breaking changes**: 2 (return types + MapBox.get)
- **Backward compatibility**: Via `HAKO_COLLECTION_V2=1` flag

---

## 💡 7. Code Examples: Before & After

### Example 1: MapBox.get() Error Handling

#### ❌ Before (Broken)
```hako
static box UserCache {
    users: MapBox

    getUser(id) {
        local user = me.users.get(id)

        // UGLY: String-based error detection! 😱
        if RegexFlow.find_from(user.toString(), "Key not found:", 0) >= 0 {
            return null  // Not found
        }

        return user  // Found
    }
}
```

#### ✅ After (Fixed)
```hako
static box UserCache {
    users: MapBox

    getUser(id) {
        local user = me.users.get(id)

        // CLEAN: Null check! 🎉
        if user == null {
            return null  // Not found
        }

        return user  // Found
    }
}
```

---

### Example 2: Consistent Size Checking

#### ❌ Before (Inconsistent)
```hako
static box DataProcessor {
    items: ArrayBox
    config: MapBox
    name: StringBox

    checkSizes() {
        local count1 = me.items.length()    // ArrayBox uses "length"
        local count2 = me.config.size()     // MapBox uses "size"
        // me.name.??? - NO SIZE METHOD! 😱

        print("Items: " + count1)
        print("Config: " + count2)
        print("Name length: ???")  // Must use workaround
    }
}
```

#### ✅ After (Unified)
```hako
static box DataProcessor {
    items: ArrayBox
    config: MapBox
    name: StringBox

    checkSizes() {
        local count1 = me.items.size()    // ✅ Unified
        local count2 = me.config.size()   // ✅ Unified
        local count3 = me.name.size()     // ✅ NEW: String size!

        print("Items: " + count1)
        print("Config: " + count2)
        print("Name length: " + count3)

        // Bonus: isEmpty() convenience
        if me.items.isEmpty() {
            print("No items!")
        }
    }
}
```

---

### Example 3: Mutation Return Values

#### ❌ Before (Messy)
```hako
static box DataManager {
    data: MapBox

    updateConfig(key, value) {
        local result = me.data.set(key, value)
        // result = StringBox("Set key: config_timeout")

        // What to do with this? Ignore it? Check it?
        print(result)  // Useless message
    }
}
```

#### ✅ After (Clean)
```hako
static box DataManager {
    data: MapBox

    updateConfig(key, value) {
        me.data.set(key, value)  // Returns null (clean!)

        // Or chain operations:
        me.data.set("key1", "val1")
        me.data.set("key2", "val2")  // No messy return values
    }
}
```

---

### Example 4: StringBox Missing Methods

#### ❌ Before (No substring!)
```hako
static box TextProcessor {
    text: StringBox

    extractPrefix(len) {
        // NO substring() method! 😱
        // Must use workarounds:

        // Workaround 1: Split and join
        local chars = me.text.split("")
        // ... complex logic ...

        // Workaround 2: Manual parsing
        local result = ""
        local i = 0
        loop(i < len) {
            // ... complex loop ...
            i = i + 1
        }

        return result
    }
}
```

#### ✅ After (Has substring!)
```hako
static box TextProcessor {
    text: StringBox

    extractPrefix(len) {
        // CLEAN: Just use substring! 🎉
        return me.text.substring(0, len)
    }

    getFirstChar() {
        // BONUS: charAt() method!
        return me.text.charAt(0)
    }

    checkSize() {
        // BONUS: size() method!
        if me.text.size() > 100 {
            print("Long text!")
        }
    }
}
```

---

## 🎯 8. Recommendation Summary

### Priority 1: Critical Fixes (Do First)

1. **MapBox.get() → Return NullBox** 🔴
   - **Impact**: HIGH (50-100 call sites)
   - **Benefit**: Eliminates string-based error detection anti-pattern
   - **Timeline**: 3-5 days

2. **Add StringBox.size()** 🔴
   - **Impact**: MEDIUM (critical missing feature)
   - **Benefit**: Enables basic string length queries
   - **Timeline**: 1 day

### Priority 2: Consistency Improvements

3. **Unify size/length naming** 🟡
   - ArrayBox.length() → ArrayBox.size()
   - Keep .length() as deprecated alias
   - **Timeline**: 1-2 days

4. **Add isEmpty() to all collections** 🟡
   - Convenience method for `size() == 0`
   - **Timeline**: 1 day

5. **Add StringBox.substring() and .charAt()** 🟡
   - Critical missing methods
   - **Timeline**: 1-2 days

### Priority 3: Polish & Cleanup

6. **Fix mutation return types** 🟢
   - set/clear/push → Return NullBox (not messages)
   - **Timeline**: 2-3 days

7. **Remove MapBox.forEach()** 🟢
   - Non-functional method (callback not executed)
   - **Timeline**: 1 day

8. **Remove deprecated methods** 🟢
   - After 1-2 release cycles
   - **Timeline**: 1 day

---

## 📅 9. Implementation Timeline

### Week 1: Non-Breaking Additions
- **Day 1-2**: Add new methods (size, isEmpty, indexOf, substring, charAt)
- **Day 3**: Add tests and documentation
- **Day 4-5**: Add deprecation warnings

### Week 2: Breaking Changes
- **Day 6-8**: Fix MapBox.get() (with feature flag)
- **Day 9**: Fix mutation return types
- **Day 10**: Final testing and documentation

### Future: Cleanup (1-2 months later)
- Remove deprecated methods
- Remove feature flags
- Update all documentation

---

## 🔍 10. Open Questions & Decisions Needed

### Q1: MapBox.delete() vs remove()?
- **Current**: `delete(key)` (matches JavaScript)
- **Alternative**: `remove(key)` (matches ArrayBox)
- **Recommendation**: **Keep `delete()`** - JavaScript Map uses it, avoids confusion

### Q2: Return deleted value or boolean?
```hako
// Option A: Return deleted value (like Rust)
local old = map.delete("key")  // → Box | NullBox

// Option B: Return success boolean (like JavaScript)
local success = map.delete("key")  // → BoolBox
```
- **Recommendation**: **Option A** (return value) - more useful, matches Rust HashMap

### Q3: Negative indices for slice/substring?
```hako
// Python-style negative indices
arr.slice(0, -1)    // All but last element
str.substring(0, -1)  // All but last char
```
- **Recommendation**: **Support negative indices** - very useful, matches Python/JS

### Q4: ArrayBox.get() type coercion?
```hako
// Should this work?
array.get("0")  // String index instead of integer
```
- **Current**: Requires IntegerBox
- **Recommendation**: **Keep strict** - fail-fast, no implicit conversion

---

## 📚 11. Related Documentation

### Update Required
- [ ] `/docs/reference/boxes-system/box-reference.md` - Update API docs
- [ ] `/docs/guides/language-guide.md` - Update collection examples
- [ ] `/docs/reference/language/quick-reference.md` - Update quick ref
- [ ] `/CLAUDE.md` - Update Box method reference

### New Documentation Needed
- [ ] Migration guide: "Upgrading to Collection API v2"
- [ ] Best practices: "Working with Collections in Hakorune"
- [ ] Troubleshooting: "Common Collection API Issues"

---

## ✅ 12. Success Criteria

### Phase 1 Complete When:
- [ ] All 10 new methods implemented and tested
- [ ] No breaking changes introduced
- [ ] Documentation updated

### Phase 2 Complete When:
- [ ] Deprecation warnings added
- [ ] Migration guide published
- [ ] All tests pass

### Phase 3-4 Complete When:
- [ ] All return types fixed
- [ ] MapBox.get() returns NullBox
- [ ] All affected code migrated
- [ ] Feature flag functional

### Final Success When:
- [ ] Zero string-based error detection in codebase
- [ ] All collections use `.size()` consistently
- [ ] All collections have `.isEmpty()`
- [ ] StringBox has `.substring()` and `.charAt()`
- [ ] All mutation methods return NullBox
- [ ] All tests pass
- [ ] All documentation updated
- [ ] User feedback positive

---

## 🎉 Conclusion

This proposal provides:

1. **Immediate value** - Add critical missing methods (StringBox.size, substring, charAt)
2. **Long-term consistency** - Unified naming and behavior across all collections
3. **Type safety** - NullBox returns instead of error strings
4. **Clear migration path** - Feature flags, deprecation warnings, batch scripts
5. **"箱らしい" design** - Simple, explicit, predictable collection APIs

**Next Steps**:
1. Review this proposal with team
2. Decide on priorities (recommend: MapBox.get() first)
3. Create implementation branch
4. Begin Phase 1 (non-breaking additions)

**Estimated Total Effort**: 1-2 weeks development + 1-2 months migration period

---

**Author**: Claude (Anthropic)
**Date**: 2025-10-09
**Status**: ⏳ Awaiting Review


### Semantics Update (Phase 15.7)

- Array.slice(start, end)
  - Profile quick-selfhost expects: when `end < 0`, clamp to `len` (full tail).
  - Core policy is centralized in `hako_core_array::slice_bounds` to keep builtin/plugin consistent.
  - Builders keep `slice` on Method path; VM invokes `hako_core_array` via builtin/plugin invokers.
