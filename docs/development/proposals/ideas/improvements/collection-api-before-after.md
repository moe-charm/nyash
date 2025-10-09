# 🔄 Collection API: Before & After

**Visual guide showing the transformation from inconsistent to unified collection APIs**

---

## 🚨 Problem 1: MapBox.get() Error String Hell

### ❌ BEFORE (Broken)

```hako
static box UserManager {
    users: MapBox

    loadUser(id) {
        local user = me.users.get(id)

        // 😱 UGLY: Check for error STRING!
        local error_check = RegexFlow.find_from(user.toString(), "Key not found:", 0)
        if error_check >= 0 {
            print("User not found")
            return null
        }

        // Or even worse:
        if user.toString().startsWith("Key not found") {
            return null
        }

        return user
    }
}
```

**Problems**:
- ❌ Error detection via string matching (fragile!)
- ❌ Forces toString() conversion (type unsafe)
- ❌ No type safety (can't distinguish error from actual data)
- ❌ Inconsistent with ArrayBox (which returns NullBox)

### ✅ AFTER (Fixed)

```hako
static box UserManager {
    users: MapBox

    loadUser(id) {
        local user = me.users.get(id)

        // ✅ CLEAN: Simple null check!
        if user == null {
            print("User not found")
            return null
        }

        return user
    }
}
```

**Benefits**:
- ✅ Type-safe null check
- ✅ Consistent with ArrayBox
- ✅ Clear intent
- ✅ "箱らしい" (Box-like) design

---

## 🚨 Problem 2: StringBox Missing Critical Methods

### ❌ BEFORE (No size/substring)

```hako
static box TextProcessor {
    text: StringBox

    // Problem 1: No size() method
    checkLength() {
        // 😱 Must use workarounds!

        // Workaround 1: Split into chars and count
        local chars = me.text.split("")
        local len = chars.length()

        // Workaround 2: Convert to array
        local arr = me.text.split("")
        if arr.length() > 100 {
            print("Too long!")
        }
    }

    // Problem 2: No substring() method
    extractPrefix(n) {
        // 😱 Complex manual implementation!
        local chars = me.text.split("")
        local result = ""
        local i = 0

        loop(i < n) {
            if i < chars.length() {
                result = result + chars.get(i)
            }
            i = i + 1
        }

        return result
    }

    // Problem 3: No charAt() method
    getFirstChar() {
        // 😱 Must split and extract
        local chars = me.text.split("")
        if chars.length() > 0 {
            return chars.get(0)
        }
        return null
    }
}
```

**Problems**:
- ❌ No direct `.size()` method
- ❌ No `.substring()` method (basic string operation!)
- ❌ No `.charAt()` method (basic string operation!)
- ❌ Forces complex workarounds with `.split("")`

### ✅ AFTER (Fixed)

```hako
static box TextProcessor {
    text: StringBox

    // Problem 1 SOLVED: Direct size() method
    checkLength() {
        // ✅ Simple and clear!
        local len = me.text.size()

        if len > 100 {
            print("Too long!")
        }
    }

    // Problem 2 SOLVED: substring() method
    extractPrefix(n) {
        // ✅ One line!
        return me.text.substring(0, n)
    }

    // Problem 3 SOLVED: charAt() method
    getFirstChar() {
        // ✅ One line!
        return me.text.charAt(0)  // Returns null if empty
    }

    // BONUS: isEmpty() convenience
    checkEmpty() {
        if me.text.isEmpty() {
            print("Empty text!")
        }
    }
}
```

**Benefits**:
- ✅ Direct `.size()` method
- ✅ Standard `.substring()` method (like JS/Java/Python)
- ✅ Standard `.charAt()` method (like Java/JS)
- ✅ Bonus: `.isEmpty()` convenience method

---

## 🚨 Problem 3: Inconsistent Naming (size vs length)

### ❌ BEFORE (Inconsistent)

```hako
static box DataStore {
    items: ArrayBox
    cache: MapBox
    name: StringBox

    showStats() {
        // 😱 Different method names for same concept!
        local item_count = me.items.length()    // ArrayBox uses "length"
        local cache_count = me.cache.size()     // MapBox uses "size"
        // local name_len = me.name.???         // StringBox has NOTHING!

        print("Items: " + item_count)
        print("Cache: " + cache_count)
        print("Name: ???")

        // 😱 Inconsistent empty checks
        if me.items.length() == 0 {
            print("No items")
        }
        if me.cache.size() == 0 {
            print("No cache")
        }
        // Can't check name length at all!
    }
}
```

**Problems**:
- ❌ ArrayBox: `.length()`
- ❌ MapBox: `.size()`
- ❌ StringBox: Nothing!
- ❌ Confusing for users (which name to use?)
- ❌ No consistency across collection types

### ✅ AFTER (Unified)

```hako
static box DataStore {
    items: ArrayBox
    cache: MapBox
    name: StringBox

    showStats() {
        // ✅ CONSISTENT: All use .size()!
        local item_count = me.items.size()
        local cache_count = me.cache.size()
        local name_len = me.name.size()

        print("Items: " + item_count)
        print("Cache: " + cache_count)
        print("Name: " + name_len)

        // ✅ CONSISTENT: All use .isEmpty()!
        if me.items.isEmpty() {
            print("No items")
        }
        if me.cache.isEmpty() {
            print("No cache")
        }
        if me.name.isEmpty() {
            print("No name")
        }
    }
}
```

**Benefits**:
- ✅ Unified `.size()` method across all collections
- ✅ Unified `.isEmpty()` convenience method
- ✅ Predictable API (learn once, use everywhere)
- ✅ "箱らしい" consistency

---

## 🚨 Problem 4: Search Method Naming

### ❌ BEFORE (Inconsistent)

```hako
static box SearchDemo {
    items: ArrayBox
    text: StringBox

    findStuff() {
        // 😱 Different method names for same operation!
        local arr_index = me.items.indexOf("target")     // ArrayBox: indexOf
        local str_index = me.text.find("pattern")        // StringBox: find

        // Confusing for users:
        // - Which collection uses indexOf?
        // - Which uses find?
        // - Are they the same operation?
    }
}
```

**Problems**:
- ❌ ArrayBox: `.indexOf()`
- ❌ StringBox: `.find()`
- ❌ Same operation, different names
- ❌ Cognitive overhead

### ✅ AFTER (Unified)

```hako
static box SearchDemo {
    items: ArrayBox
    text: StringBox

    findStuff() {
        // ✅ CONSISTENT: Both use .indexOf()!
        local arr_index = me.items.indexOf("target")
        local str_index = me.text.indexOf("pattern")

        // Both return -1 if not found (consistent!)
        if arr_index == -1 {
            print("Not in array")
        }
        if str_index == -1 {
            print("Not in string")
        }
    }
}
```

**Benefits**:
- ✅ Unified `.indexOf()` method
- ✅ Same return convention (-1 for not found)
- ✅ Matches JavaScript/Java naming
- ✅ Keep `.find()` as deprecated alias for backward compatibility

---

## 🚨 Problem 5: Mutation Return Values

### ❌ BEFORE (Messy)

```hako
static box ConfigManager {
    settings: MapBox
    values: ArrayBox

    update() {
        // 😱 Useless return messages!
        local r1 = me.settings.set("timeout", 30)
        // r1 = StringBox("Set key: timeout")

        local r2 = me.settings.clear()
        // r2 = StringBox("Map cleared")

        local r3 = me.values.push(42)
        // r3 = StringBox("ok")

        // What to do with these?
        // - Ignore them? (why return at all?)
        // - Check them? (how?)
        // - Print them? (useless noise)

        print(r1)  // "Set key: timeout" (useless!)
        print(r2)  // "Map cleared" (useless!)
        print(r3)  // "ok" (useless!)
    }
}
```

**Problems**:
- ❌ Returns useless success messages
- ❌ No actual value returned
- ❌ Encourages ignoring return values
- ❌ Inconsistent with modern language design

### ✅ AFTER (Clean)

```hako
static box ConfigManager {
    settings: MapBox
    values: ArrayBox

    update() {
        // ✅ Clean: mutations return null
        me.settings.set("timeout", 30)
        me.settings.clear()
        me.values.push(42)

        // Or chain operations:
        me.settings
            .set("key1", "val1")
            .set("key2", "val2")  // Wait... this needs builder pattern

        // For now, just sequential:
        me.settings.set("key1", "val1")
        me.settings.set("key2", "val2")
    }

    // DELETE returns deleted value (useful!)
    removeOldConfig(key) {
        local old_value = me.settings.delete(key)
        if old_value != null {
            print("Removed: " + old_value.toString())
        }
    }
}
```

**Benefits**:
- ✅ Mutations return `NullBox` (clean, no noise)
- ✅ `delete()` returns deleted value (useful!)
- ✅ Matches Rust HashMap behavior
- ✅ Clear intent (mutation, not query)

---

## 📊 Summary: Before & After

### API Consistency Matrix

| Operation | ArrayBox Before | MapBox Before | StringBox Before | **After (All)** |
|-----------|----------------|---------------|------------------|-----------------|
| **Size** | `.length()` | `.size()` | ❌ Missing | ✅ `.size()` |
| **Empty Check** | `size()==0` | `size()==0` | ❌ N/A | ✅ `.isEmpty()` |
| **Get Missing** | NullBox ✅ | StringBox(error) 😱 | N/A | ✅ `NullBox` |
| **Search** | `.indexOf()` | N/A | `.find()` | ✅ `.indexOf()` |
| **Mutation Return** | StringBox(msg) | StringBox(msg) | N/A | ✅ `NullBox` |
| **Substring/Slice** | `.slice()` ✅ | N/A | ❌ Missing | ✅ `.substring()` |
| **Char Access** | `.get()` ✅ | N/A | ❌ Missing | ✅ `.charAt()` |

### Key Improvements

1. **Type Safety** ✅
   - MapBox.get() returns NullBox (not error string)
   - Consistent null handling across all collections

2. **Method Naming** ✅
   - Unified `.size()` (not length/size mix)
   - Unified `.indexOf()` (not indexOf/find mix)

3. **Completeness** ✅
   - StringBox gains `.size()`, `.substring()`, `.charAt()`
   - All collections gain `.isEmpty()`

4. **Clean Returns** ✅
   - Mutations return NullBox (not messages)
   - delete() returns deleted value (useful!)

5. **"箱らしい" Design** ✅
   - Simple, explicit, predictable
   - Learn once, use everywhere
   - Consistent with Rust Option<T> pattern

---

## 🎯 Migration Path

### Phase 1: Add New Methods (No Breaking Changes)
```hako
// Add to all collections
.size()           // Unified size method
.isEmpty()        // Convenience method

// Add to StringBox
.indexOf()        // Search (alias for .find())
.substring()      // Extract substring
.charAt()         // Get character
```

### Phase 2: Fix Critical Bug (Breaking - With Feature Flag)
```hako
// MapBox.get() behavior change
HAKO_COLLECTION_V2=1  // Enable new behavior

// BEFORE:
map.get("missing")  // → StringBox("Key not found: missing")

// AFTER:
map.get("missing")  // → NullBox
```

### Phase 3: Fix Return Types (Semi-Breaking - With Feature Flag)
```hako
// Mutation methods return null
map.set(k, v)    // → NullBox (not message)
map.clear()      // → NullBox (not message)
array.push(x)    // → NullBox (not message)
map.delete(k)    // → deleted_value | NullBox (useful!)
```

### Phase 4: Cleanup (After 1-2 Releases)
```hako
// Remove deprecated methods
- ArrayBox.length()  (use .size())
- StringBox.find()   (use .indexOf())
- MapBox.forEach()   (non-functional)
```

---

## 📚 Related Documents

- **Full Proposal**: [unified-collection-interface-proposal.md](./unified-collection-interface-proposal.md)
- **Quick Fixes**: [collection-api-quick-fixes.md](./collection-api-quick-fixes.md)
- **Comparison Table**: [collection-api-comparison-table.md](./collection-api-comparison-table.md)

---

**Author**: Claude (Anthropic)
**Date**: 2025-10-09
**Status**: ⏳ Awaiting Implementation
