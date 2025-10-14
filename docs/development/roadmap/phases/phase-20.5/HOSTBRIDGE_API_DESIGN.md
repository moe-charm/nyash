# HostBridge API Design — Phase 20.5

**Purpose**: Define minimal C-ABI surface for Rust↔Hakorune boundary
**Status**: Design (Week 1-4 implementation)
**Platforms**: Ubuntu/Windows

---

## 🎯 Design Principles

### 1. Minimal Surface
- **5 core functions** (not 50)
- Each function has single responsibility
- No feature creep

### 2. Platform Independence
- Pure C-ABI (no C++ exceptions, no OS-specific types)
- Works on Ubuntu (gcc/clang) and Windows (MSVC/MinGW)
- Thread-Local Storage (TLS) for error handling

### 3. Handle-Based Ownership
- All Hakorune values represented as `HakoHandle` (opaque 64-bit)
- Explicit `Retain/Release` for reference counting
- No manual memory management exposed

### 4. UTF-8 Everywhere
- All strings are UTF-8 (`const char*` or `HakoStr`)
- No wide strings (Windows `wchar_t`)
- Simple, consistent encoding

### 5. Fail-Fast
- Errors are explicit (return codes + `Hako_LastError`)
- No silent failures
- No undefined behavior

---

## 📋 Core API (5 Functions)

### 1. `Hako_RunScriptUtf8`

**Purpose**: Execute Hakorune script, return handle to result

```c
int32_t Hako_RunScriptUtf8(
    const char* source_utf8,
    HakoHandle* out_result_handle
);
```

**Parameters**:
- `source_utf8`: Hakorune source code (null-terminated UTF-8)
- `out_result_handle`: Output handle to result value

**Returns**:
- `0`: Success (`out_result_handle` is valid)
- `-1`: Error (use `Hako_LastError()` for details)

**Ownership**:
- Caller owns `out_result_handle` (must call `Hako_Release`)
- Caller manages `source_utf8` memory

**Example**:
```c
const char* script = "static box Main { main() { return 42 } }";
HakoHandle result = 0;
if (Hako_RunScriptUtf8(script, &result) == 0) {
    // Success - result is valid
    // ... use result ...
    Hako_Release(result);
} else {
    fprintf(stderr, "Error: %s\n", Hako_LastError());
}
```

---

### 2. `Hako_Retain`

**Purpose**: Increment reference count on handle

```c
void Hako_Retain(HakoHandle handle);
```

**Parameters**:
- `handle`: Handle to retain

**Returns**: void (no error possible)

**Ownership**:
- Caller must call `Hako_Release` for each `Retain`

**Example**:
```c
HakoHandle result = /* from Hako_RunScriptUtf8 */;
Hako_Retain(result);  // Now refcount = 2
// Pass to another function
other_function(result);
// Still valid here
Hako_Release(result);  // Decrement to 1
Hako_Release(result);  // Decrement to 0, freed
```

---

### 3. `Hako_Release`

**Purpose**: Decrement reference count, free if zero

```c
void Hako_Release(HakoHandle handle);
```

**Parameters**:
- `handle`: Handle to release (can be 0/null)

**Returns**: void (no error possible)

**Ownership**:
- Handle may be freed if refcount reaches zero
- Safe to call with `handle=0` (no-op)

**Example**:
```c
HakoHandle result = /* from Hako_RunScriptUtf8 */;
Hako_Release(result);  // Decrement refcount
// result is now INVALID - do not use
```

---

### 4. `Hako_ToUtf8`

**Purpose**: Get UTF-8 string view of handle

```c
typedef struct {
    const char* data;  // UTF-8 bytes (NOT null-terminated)
    size_t len;        // Byte length
} HakoStr;

int32_t Hako_ToUtf8(
    HakoHandle handle,
    HakoStr* out_str
);
```

**Parameters**:
- `handle`: Handle to convert to string
- `out_str`: Output string view

**Returns**:
- `0`: Success (`out_str` is valid)
- `-1`: Error (not a string, or conversion failed)

**Ownership**:
- `out_str->data` is **borrowed** (valid until `Hako_Release(handle)`)
- Caller must NOT free `out_str->data`
- String is NOT null-terminated (use `len`)

**Example**:
```c
HakoHandle result = /* from Hako_RunScriptUtf8 */;
HakoStr str;
if (Hako_ToUtf8(result, &str) == 0) {
    printf("Result: %.*s\n", (int)str.len, str.data);
    // Do NOT free str.data
}
Hako_Release(result);  // Now str.data is INVALID
```

---

### 5. `Hako_LastError`

**Purpose**: Get last error message (thread-local)

```c
const char* Hako_LastError(void);
```

**Parameters**: none

**Returns**:
- Error message (null-terminated UTF-8)
- Valid until next Hako API call on this thread
- Never returns NULL (returns "Unknown error" if none)

**Thread-Safety**:
- Uses Thread-Local Storage (TLS)
- Each thread has independent error state

**Example**:
```c
if (Hako_RunScriptUtf8(script, &result) != 0) {
    fprintf(stderr, "Error: %s\n", Hako_LastError());
}
```

---

## 🔧 Optional Functions (Future)

### `Hako_ApiVersion`

**Purpose**: Get API version for compatibility checks

```c
typedef struct {
    uint32_t major;  // Breaking changes
    uint32_t minor;  // New features
    uint32_t patch;  // Bug fixes
} HakoVersion;

HakoVersion Hako_ApiVersion(void);
```

**Example**:
```c
HakoVersion ver = Hako_ApiVersion();
if (ver.major != 1) {
    fprintf(stderr, "Incompatible API version: %u.%u.%u\n",
            ver.major, ver.minor, ver.patch);
    exit(1);
}
```

### `Hako_ToInt64`

**Purpose**: Get integer value from handle

```c
int32_t Hako_ToInt64(
    HakoHandle handle,
    int64_t* out_value
);
```

**Example**:
```c
int64_t num;
if (Hako_ToInt64(result, &num) == 0) {
    printf("Result: %lld\n", (long long)num);
}
```

---

## 🏗️ Implementation Strategy

### Phase 1: Rust Side (Week 1-2)

**File**: `src/hostbridge/mod.rs` (new)

```rust
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

pub type HakoHandle = u64;

#[repr(C)]
pub struct HakoStr {
    pub data: *const u8,
    pub len: usize,
}

#[no_mangle]
pub extern "C" fn Hako_RunScriptUtf8(
    source_utf8: *const c_char,
    out_result_handle: *mut HakoHandle
) -> c_int {
    // 1. Convert C string to Rust &str
    // 2. Parse + execute Hakorune code
    // 3. Create handle for result
    // 4. Store in HandleRegistry
    // 5. Return handle via out_result_handle
    unimplemented!("Week 1-2")
}

#[no_mangle]
pub extern "C" fn Hako_Retain(handle: HakoHandle) {
    // Increment refcount in HandleRegistry
    unimplemented!("Week 1-2")
}

#[no_mangle]
pub extern "C" fn Hako_Release(handle: HakoHandle) {
    // Decrement refcount in HandleRegistry
    // Free if refcount == 0
    unimplemented!("Week 1-2")
}

#[no_mangle]
pub extern "C" fn Hako_ToUtf8(
    handle: HakoHandle,
    out_str: *mut HakoStr
) -> c_int {
    // 1. Lookup handle in HandleRegistry
    // 2. Convert to string (if possible)
    // 3. Return borrowed pointer + length
    unimplemented!("Week 1-2")
}

thread_local! {
    static LAST_ERROR: RefCell<CString> = RefCell::new(CString::new("").unwrap());
}

#[no_mangle]
pub extern "C" fn Hako_LastError() -> *const c_char {
    LAST_ERROR.with(|err| err.borrow().as_ptr())
}

fn set_last_error(msg: &str) {
    LAST_ERROR.with(|err| {
        *err.borrow_mut() = CString::new(msg).unwrap_or_default();
    });
}
```

### Phase 2: HandleRegistry (Week 2-3)

**File**: `src/hostbridge/handle_registry.rs` (new)

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct HandleRegistry {
    next_id: u64,
    handles: HashMap<u64, (Arc<Box<dyn Any>>, u32)>,  // (value, refcount)
}

impl HandleRegistry {
    pub fn new() -> Self {
        HandleRegistry {
            next_id: 1,
            handles: HashMap::new(),
        }
    }

    pub fn insert(&mut self, value: Arc<Box<dyn Any>>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.handles.insert(id, (value, 1));  // refcount = 1
        id
    }

    pub fn retain(&mut self, handle: u64) {
        if let Some((_, ref mut refcount)) = self.handles.get_mut(&handle) {
            *refcount += 1;
        }
    }

    pub fn release(&mut self, handle: u64) {
        if let Some((_, ref mut refcount)) = self.handles.get_mut(&handle) {
            *refcount -= 1;
            if *refcount == 0 {
                self.handles.remove(&handle);
            }
        }
    }

    pub fn get(&self, handle: u64) -> Option<Arc<Box<dyn Any>>> {
        self.handles.get(&handle).map(|(val, _)| val.clone())
    }
}

lazy_static! {
    static ref GLOBAL_REGISTRY: Mutex<HandleRegistry> = Mutex::new(HandleRegistry::new());
}
```

### Phase 3: C Header (Week 3)

**File**: `include/hakorune_hostbridge.h` (new)

```c
#ifndef HAKORUNE_HOSTBRIDGE_H
#define HAKORUNE_HOSTBRIDGE_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque handle type */
typedef uint64_t HakoHandle;

/* String view (not null-terminated) */
typedef struct {
    const char* data;
    size_t len;
} HakoStr;

/* Core API */
int32_t Hako_RunScriptUtf8(const char* source_utf8, HakoHandle* out_result_handle);
void Hako_Retain(HakoHandle handle);
void Hako_Release(HakoHandle handle);
int32_t Hako_ToUtf8(HakoHandle handle, HakoStr* out_str);
const char* Hako_LastError(void);

/* Optional API */
typedef struct {
    uint32_t major;
    uint32_t minor;
    uint32_t patch;
} HakoVersion;

HakoVersion Hako_ApiVersion(void);
int32_t Hako_ToInt64(HakoHandle handle, int64_t* out_value);

#ifdef __cplusplus
}
#endif

#endif /* HAKORUNE_HOSTBRIDGE_H */
```

### Phase 4: ABI Tests (Week 4)

**File**: `tests/hostbridge_abi_test.c`

```c
#include "hakorune_hostbridge.h"
#include <stdio.h>
#include <assert.h>

void test_hello_world() {
    const char* script = "static box Main { main() { return \"Hello\" } }";
    HakoHandle result = 0;

    assert(Hako_RunScriptUtf8(script, &result) == 0);
    assert(result != 0);

    HakoStr str;
    assert(Hako_ToUtf8(result, &str) == 0);
    assert(str.len == 5);
    assert(memcmp(str.data, "Hello", 5) == 0);

    Hako_Release(result);
    printf("✅ test_hello_world PASS\n");
}

void test_retain_release() {
    const char* script = "static box Main { main() { return 42 } }";
    HakoHandle result = 0;

    assert(Hako_RunScriptUtf8(script, &result) == 0);
    Hako_Retain(result);  // refcount = 2
    Hako_Release(result);  // refcount = 1

    // Still valid
    int64_t num;
    assert(Hako_ToInt64(result, &num) == 0);
    assert(num == 42);

    Hako_Release(result);  // refcount = 0, freed
    printf("✅ test_retain_release PASS\n");
}

void test_error_handling() {
    const char* bad_script = "this is not valid syntax!!!";
    HakoHandle result = 0;

    assert(Hako_RunScriptUtf8(bad_script, &result) != 0);
    const char* err = Hako_LastError();
    assert(err != NULL);
    assert(strlen(err) > 0);

    printf("✅ test_error_handling PASS (error: %s)\n", err);
}

int main() {
    test_hello_world();
    test_retain_release();
    test_error_handling();
    printf("✅ All HostBridge ABI tests PASS\n");
    return 0;
}
```

**Build**:
```bash
# Ubuntu
gcc -o test_abi tests/hostbridge_abi_test.c -L target/release -lhakorune_kernel
./test_abi

# Windows (MinGW)
gcc -o test_abi.exe tests/hostbridge_abi_test.c -L target/release -lhako_kernel
./test_abi.exe

# Windows (MSVC)
cl tests/hostbridge_abi_test.c /link target/release/hako_kernel.lib
./test_abi.exe
```

---

## 🧪 Testing Strategy

### Level 1: Unit Tests (Rust)

**File**: `src/hostbridge/tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_registry_insert_release() {
        let mut registry = HandleRegistry::new();
        let value = Arc::new(Box::new(42i64) as Box<dyn Any>);
        let handle = registry.insert(value);

        assert!(registry.get(handle).is_some());
        registry.release(handle);
        assert!(registry.get(handle).is_none());
    }

    #[test]
    fn test_handle_registry_retain() {
        let mut registry = HandleRegistry::new();
        let value = Arc::new(Box::new("test") as Box<dyn Any>);
        let handle = registry.insert(value);

        registry.retain(handle);
        registry.release(handle);
        assert!(registry.get(handle).is_some());  // Still valid
        registry.release(handle);
        assert!(registry.get(handle).is_none());  // Now freed
    }
}
```

### Level 2: ABI Tests (C)

**Ubuntu + Windows** (see Phase 4 above)

**Coverage**:
- Basic execution (hello world)
- Handle lifetime (retain/release)
- Error handling (syntax errors)
- String conversion (UTF-8)
- Integer conversion (Int64)

### Level 3: Integration Tests (Hakorune)

**File**: `apps/examples/hostbridge/test_hostbridge_call.hako`

```hakorune
using hostbridge.HostBridgeBox

static box Main {
  main() {
    local bridge = new HostBridgeBox()

    // Test 1: Execute script via C-ABI
    local script = "static box Main { main() { return 42 } }"
    local result = bridge.run_script(script)

    if (result != 42) {
      bridge.log("❌ FAIL: Expected 42")
      return 1
    }

    bridge.log("✅ PASS: HostBridge call works")
    return 0
  }
}
```

---

## ⚙️ Error Handling Strategy

### Error Categories

1. **Parse Errors**: Invalid syntax
   - Example: `Hako_LastError()` → "Parse error at line 3: Expected '}'"

2. **Execution Errors**: Runtime exceptions
   - Example: `Hako_LastError()` → "Runtime error: Division by zero"

3. **Type Errors**: Invalid type conversion
   - Example: `Hako_ToInt64(string_handle, &num)` → -1
   - `Hako_LastError()` → "Type error: Expected Int, got String"

4. **Handle Errors**: Invalid handle access
   - Example: `Hako_Release(999999)` → silently ignored (no error)
   - `Hako_ToUtf8(999999, &str)` → -1, "Invalid handle"

### Error Message Format

```
[Category] Context: Details

Examples:
[Parse] Line 3: Expected '}'
[Runtime] Division by zero in function 'calculate'
[Type] Expected Int, got String
[Handle] Invalid handle: 12345
```

---

## 🛡️ Platform Considerations

### Ubuntu (gcc/clang)

**Thread-Local Storage**:
```c
__thread CString LAST_ERROR;  // gcc extension
```

**Linker Flags**:
```bash
-lpthread -ldl -lm
```

### Windows (MSVC)

**Thread-Local Storage**:
```c
__declspec(thread) CString LAST_ERROR;  // MSVC extension
```

**Linker Flags**:
```
/link advapi32.lib ws2_32.lib
```

### Windows (MinGW)

**Thread-Local Storage**:
```c
__thread CString LAST_ERROR;  // gcc extension works
```

**Linker Flags**:
```bash
-lws2_32 -ladvapi32
```

---

## 📋 Success Criteria

### Functional

- [ ] All 5 core functions implemented
- [ ] ABI tests PASS on Ubuntu
- [ ] ABI tests PASS on Windows
- [ ] Error handling works (TLS)
- [ ] Handle lifecycle correct (no leaks)

### Non-Functional

- [ ] API documentation complete
- [ ] Examples for each function
- [ ] Integration tests in Hakorune
- [ ] Performance: < 1ms overhead per call

---

## 🔗 Related Documents

- [STRATEGY_RECONCILIATION.md](STRATEGY_RECONCILIATION.md) - Why HostBridge?
- [PURE_HAKORUNE_ROADMAP.md](PURE_HAKORUNE_ROADMAP.md) - Overall plan
- [OP_EQ_MIGRATION.md](OP_EQ_MIGRATION.md) - Next step after HostBridge

---

**Status**: Design Phase (Week 1-4 implementation)
**Owner**: ChatGPT (implementation), Claude (review)
**Timeline**: Week 1-4 of Phase 20.5
