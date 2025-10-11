# Hakorune Shared ABI Architecture Design

**Date**: 2025-10-11
**Status**: Design Proposal
**Author**: Claude (Architecture Analysis)

## Executive Summary

This document proposes a 3-layer ABI architecture that:
1. **Eliminates code duplication** between nyash_kernel and plugins (現在の2,387行を共有)
2. **Breaks circular dependency** between nyash_kernel ↔ nyash-rust
3. **Enables future C ABI migration** for LLVM-generated code
4. **Maintains single source of truth** for all ABI functions

**User's Vision**: "めちゃくちゃ使いたい！" - Share code WITHOUT maintaining 2 places that causes "禿げます" (hair loss)!

---

## Current State Analysis

### Problem 1: Code Duplication
```
nyash_kernel/src/plugin/array.rs (156 lines)
plugins/nyash-array-plugin/src/lib.rs (564 lines)
```

**Issue**: Both implement similar logic for Array operations but cannot share code.

### Problem 2: Circular Dependency
```
nyash_kernel
  ├─ depends on: nyash-rust (concrete Box types)
  └─ provides: C ABI functions

nyash-rust
  └─ (should not depend on nyash_kernel)
```

**Issue**: nyash_kernel imports `nyash_rust::boxes::array::ArrayBox`, creating tight coupling.

### Problem 3: Plugin Isolation
- Each plugin reimplements TLV encoding/decoding (100-200 lines each)
- No shared code for common operations (array bounds checking, handle management)
- Example: `hako_core_array::classify_set_index` is used by plugin but not by nyash_kernel

### Current Architecture Stats
- **nyash_kernel ABI functions**: 2,387 lines
- **hako_core_* helper crates**: 4 crates (array, map, string, callable)
- **Plugins**: 15+ plugins, each with custom TLV handling

---

## Research: C ABI Compatibility Patterns

### Industry Standards (wasmtime, v8, etc.)

#### Pattern 1: Opaque Handle Types
```rust
// C header (generated)
typedef struct WasmObject WasmObject;
WasmObject* wasm_object_new();
void wasm_object_set(WasmObject* obj, int64_t key, int64_t val);

// Rust implementation
#[repr(C)]
pub struct WasmObject {
    _private: [u8; 0],  // Zero-sized, forces opacity
}

#[no_mangle]
pub extern "C" fn wasm_object_new() -> *mut WasmObject {
    Box::into_raw(Box::new(ActualImpl::new())) as *mut WasmObject
}
```

#### Pattern 2: Pure C ABI Layer + Rust Implementation
```
Layer 1: C ABI definitions (header-only, no types)
    ↓
Layer 2: Rust implementation (uses concrete types)
    ↓
Layer 3: Thin C wrapper (calls Rust, exports C symbols)
```

#### Pattern 3: Handle Registry (Current Hakorune Approach)
```rust
// Already implemented in Hakorune!
pub fn to_handle_arc(arc: Arc<dyn NyashBox>) -> u64;
pub fn get(h: u64) -> Option<Arc<dyn NyashBox>>;
```

**Finding**: Hakorune's `host_handles` system is PERFECT for this! Just need to reorganize layers.

---

## Proposed Architecture: 3-Layer Design

### Layer 1: Pure ABI Definitions (`hako_abi` crate)

**Purpose**: C-compatible interface definitions WITHOUT Rust types.

```
crates/hako_abi/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── handles.rs      // Handle type definitions
│   ├── types.rs        // TLV tags, error codes
│   └── functions.rs    // C function signatures (trait-based)
└── include/
    └── hako_abi.h      // Generated C header (future)
```

**Key Properties**:
- **Zero dependencies** (no nyash-rust, no hako_core_*)
- **Pure trait definitions** for ABI contracts
- **C-compatible types only** (i64, u64, *const u8, etc.)

#### Example Code
```rust
// crates/hako_abi/src/handles.rs
/// Opaque handle type (u64 internally)
pub type HakoHandle = u64;
pub const HAKO_INVALID_HANDLE: HakoHandle = 0;

// crates/hako_abi/src/types.rs
/// TLV type tags (shared by all implementations)
pub const TLV_TAG_I64: u8 = 3;
pub const TLV_TAG_STRING: u8 = 6;
pub const TLV_TAG_PLUGIN_HANDLE: u8 = 8;
pub const TLV_TAG_HOST_HANDLE: u8 = 9;

/// Error codes (standardized)
pub const HAKO_SUCCESS: i32 = 0;
pub const HAKO_E_SHORT_BUFFER: i32 = -1;
pub const HAKO_E_INVALID_ARGS: i32 = -2;

// crates/hako_abi/src/functions.rs
/// Array ABI contract (trait for multiple implementations)
pub trait ArrayAbi {
    /// Create new array, returns handle
    fn array_new() -> HakoHandle;

    /// Get element at index
    fn array_get(handle: HakoHandle, idx: i64) -> i64;

    /// Set element at index
    fn array_set(handle: HakoHandle, idx: i64, val: i64) -> i64;

    /// Push element
    fn array_push(handle: HakoHandle, val: i64) -> i64;

    /// Get length
    fn array_len(handle: HakoHandle) -> i64;
}
```

---

### Layer 2: Shared Implementation (`hako_abi_impl` crate)

**Purpose**: Rust implementation using concrete types, shared by core AND plugins.

```
crates/hako_abi_impl/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── array_impl.rs   // ArrayAbi implementation
│   ├── map_impl.rs     // MapAbi implementation
│   ├── string_impl.rs  // StringAbi implementation
│   └── tlv/
│       ├── mod.rs
│       ├── encode.rs   // Shared TLV encoding
│       └── decode.rs   // Shared TLV decoding
└── tests/
```

**Dependencies**:
```toml
[dependencies]
hako_abi = { path = "../hako_abi" }
hako_core_array = { path = "../hako_core_array" }
hako_core_map = { path = "../hako_core_map" }
hako_core_string = { path = "../hako_core_string" }
# NO dependency on nyash-rust! (breaks circular dependency)
```

#### Example Code
```rust
// crates/hako_abi_impl/src/array_impl.rs
use hako_abi::{ArrayAbi, HakoHandle, HAKO_SUCCESS};
use hako_core_array::{classify_set_index, SetIndex};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}};

/// Internal array storage (plugin-side, no NyashBox dependency)
#[derive(Clone)]
pub enum ArrayValue {
    I64(i64),
    Str(String),
    Handle(u32, u32),      // Plugin handle
    HostHandle(u64),       // Host handle (for core integration)
}

struct ArrayInstance {
    data: Vec<ArrayValue>,
}

/// Thread-safe instance registry
pub struct ArrayRegistry {
    next_id: AtomicU64,
    instances: Mutex<HashMap<u64, ArrayInstance>>,
}

impl ArrayRegistry {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            instances: Mutex::new(HashMap::new()),
        }
    }

    pub fn alloc(&self) -> HakoHandle {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let mut map = self.instances.lock().unwrap();
        map.insert(id, ArrayInstance { data: Vec::new() });
        id
    }

    pub fn get_instance(&self, handle: HakoHandle) -> Option<Arc<Mutex<ArrayInstance>>> {
        let map = self.instances.lock().unwrap();
        map.get(&handle).map(|inst| Arc::new(Mutex::new(inst.clone())))
    }
}

impl ArrayAbi for ArrayRegistry {
    fn array_new() -> HakoHandle {
        REGISTRY.alloc()
    }

    fn array_get(handle: HakoHandle, idx: i64) -> i64 {
        if let Some(inst) = REGISTRY.get_instance(handle) {
            let inst = inst.lock().unwrap();
            if let Some(i) = hako_core_array::safe_get_index(inst.data.len(), idx) {
                match &inst.data[i] {
                    ArrayValue::I64(v) => *v,
                    _ => 0, // TODO: Handle other types
                }
            } else {
                0
            }
        } else {
            0
        }
    }

    fn array_set(handle: HakoHandle, idx: i64, val: i64) -> i64 {
        if let Some(inst) = REGISTRY.get_instance(handle) {
            let mut inst = inst.lock().unwrap();
            match classify_set_index(inst.data.len(), idx) {
                SetIndex::Replace(i) => {
                    inst.data[i] = ArrayValue::I64(val);
                    HAKO_SUCCESS as i64
                }
                SetIndex::Append => {
                    inst.data.push(ArrayValue::I64(val));
                    HAKO_SUCCESS as i64
                }
                SetIndex::Oob => 0,
            }
        } else {
            0
        }
    }

    fn array_push(handle: HakoHandle, val: i64) -> i64 {
        if let Some(inst) = REGISTRY.get_instance(handle) {
            let mut inst = inst.lock().unwrap();
            inst.data.push(ArrayValue::I64(val));
            inst.data.len() as i64
        } else {
            0
        }
    }

    fn array_len(handle: HakoHandle) -> i64 {
        if let Some(inst) = REGISTRY.get_instance(handle) {
            let inst = inst.lock().unwrap();
            hako_core_array::length(inst.data.len())
        } else {
            0
        }
    }
}

static REGISTRY: once_cell::sync::Lazy<ArrayRegistry> =
    once_cell::sync::Lazy::new(ArrayRegistry::new);
```

---

### Layer 3a: Core Integration (`nyash_kernel` refactored)

**Purpose**: Bridges shared implementation to nyash-rust's `host_handles` system.

```rust
// crates/hako_kernel/src/plugin/array.rs (AFTER refactoring)
use hako_abi_impl::ArrayRegistry;
use nyash_rust::runtime::host_handles;

#[no_mangle]
pub extern "C" fn nyash_array_new_h() -> i64 {
    use nyash_rust::{box_trait::NyashBox, boxes::array::ArrayBox};
    let arc: Arc<dyn NyashBox> = Arc::new(ArrayBox::new());
    host_handles::to_handle_arc(arc) as i64
}

#[no_mangle]
pub extern "C" fn nyash_array_get_h(handle: i64, idx: i64) -> i64 {
    use nyash_rust::runtime::host_handles;
    if let Some(obj) = host_handles::get(handle as u64) {
        if let Some(arr) = obj.as_any().downcast_ref::<nyash_rust::boxes::array::ArrayBox>() {
            // Delegate to concrete implementation
            let val = arr.get(Box::new(IntegerBox::new(idx)));
            if let Some(ib) = val.as_any().downcast_ref::<IntegerBox>() {
                return ib.value;
            }
        }
    }
    0
}

// Alternative: Use shared implementation for validation/bounds checking
#[no_mangle]
pub extern "C" fn nyash_array_bounds_check(len: i64, idx: i64) -> i64 {
    // Reuse hako_core_array logic!
    if hako_core_array::safe_get_index(len as usize, idx).is_some() {
        1
    } else {
        0
    }
}
```

**Result**: nyash_kernel is now much thinner, delegates to:
- `hako_core_*` for pure logic (bounds checking, etc.)
- `nyash_rust::boxes` for actual Box operations
- NO duplication of validation logic!

---

### Layer 3b: Plugin Integration (plugins refactored)

**Purpose**: Plugins use shared implementation directly.

```rust
// plugins/nyash-array-plugin/src/lib.rs (AFTER refactoring)
use hako_abi::{ArrayAbi, HakoHandle};
use hako_abi_impl::ArrayRegistry;

static REGISTRY: once_cell::sync::Lazy<ArrayRegistry> =
    once_cell::sync::Lazy::new(ArrayRegistry::new);

extern "C" fn array_invoke_id(
    instance_id: u32,
    method_id: u32,
    args: *const u8,
    args_len: usize,
    result: *mut u8,
    result_len: *mut usize,
) -> i32 {
    match method_id {
        METHOD_BIRTH => {
            let handle = ArrayRegistry::array_new();
            // Convert to TLV using shared encoder
            hako_abi_impl::tlv::write_handle(handle, result, result_len)
        }
        METHOD_GET => {
            let idx = hako_abi_impl::tlv::read_arg_i64(args, args_len, 0)?;
            let val = ArrayRegistry::array_get(instance_id as u64, idx);
            hako_abi_impl::tlv::write_i64(val, result, result_len)
        }
        METHOD_SET => {
            let idx = hako_abi_impl::tlv::read_arg_i64(args, args_len, 0)?;
            let val = hako_abi_impl::tlv::read_arg_i64(args, args_len, 1)?;
            ArrayRegistry::array_set(instance_id as u64, idx, val);
            hako_abi_impl::tlv::write_success(result, result_len)
        }
        // ... etc
    }
}
```

**Result**: Plugin code reduced by ~200 lines, all validation/logic shared!

---

## Future: C ABI Layer (Phase 3)

### Generated C Header (for LLVM-generated code)

```c
// Generated from hako_abi crate
#ifndef HAKO_ABI_H
#define HAKO_ABI_H

#include <stdint.h>

typedef uint64_t HakoHandle;

#define HAKO_INVALID_HANDLE 0
#define HAKO_SUCCESS 0
#define HAKO_E_INVALID_ARGS -2

// Array operations
HakoHandle hako_array_new(void);
int64_t hako_array_get(HakoHandle handle, int64_t idx);
int64_t hako_array_set(HakoHandle handle, int64_t idx, int64_t val);
int64_t hako_array_push(HakoHandle handle, int64_t val);
int64_t hako_array_len(HakoHandle handle);

// Map operations
HakoHandle hako_map_new(void);
// ...

#endif
```

### LLVM-Generated Code Integration

```llvm
; LLVM IR (generated by Hakorune compiler)
declare i64 @hako_array_new()
declare i64 @hako_array_get(i64 %handle, i64 %idx)
declare i64 @hako_array_set(i64 %handle, i64 %idx, i64 %val)

define i64 @my_hakorune_function() {
  %arr = call i64 @hako_array_new()
  call i64 @hako_array_push(i64 %arr, i64 42)
  %val = call i64 @hako_array_get(i64 %arr, i64 0)
  ret i64 %val
}
```

**Key Point**: Same C symbols can be:
- Implemented by Rust (Layer 2)
- Called from LLVM-generated code
- Used by plugins

**Result**: One implementation, three consumers!

---

## Migration Path

### Phase 1: Break Circular Dependency (Immediate, 1-2 days)

**Goal**: Create `hako_abi` and `hako_abi_impl` crates.

**Steps**:
1. Create `crates/hako_abi/` with trait definitions
2. Create `crates/hako_abi_impl/` with shared implementation
3. Extract TLV encoding/decoding from plugins into `hako_abi_impl::tlv`
4. Add `hako_abi_impl` as dependency to ONE plugin (proof of concept)

**Success Criteria**:
- `hako_abi` has ZERO dependencies
- `hako_abi_impl` does NOT depend on `nyash-rust`
- ONE plugin successfully uses shared implementation

**Estimated Effort**: 8-12 hours

---

### Phase 2: Unify Plugins (1-2 weeks)

**Goal**: Migrate all plugins to use `hako_abi_impl`.

**Steps**:
1. Refactor `nyash-array-plugin` (pilot)
2. Refactor `nyash-map-plugin`
3. Refactor `nyash-string-plugin`
4. Refactor remaining plugins (json, file, net, etc.)
5. Delete duplicated code (expect ~500-800 line reduction)

**Success Criteria**:
- All plugins import `hako_abi_impl`
- TLV encoding/decoding is centralized
- Validation logic uses `hako_core_*` helpers
- Tests pass for all plugins

**Estimated Effort**: 40-80 hours (2-4 hours per plugin × 15 plugins)

---

### Phase 3: Add C ABI Layer (Future, 2-4 weeks)

**Goal**: Enable LLVM-generated code to call ABI functions.

**Steps**:
1. Generate C header from `hako_abi` trait definitions
2. Add C wrapper functions that call Rust implementation
3. Export symbols with C linkage (`#[export_name = "..."]`)
4. Test from hand-written LLVM IR
5. Integrate with Hakorune's LLVM backend

**Success Criteria**:
- `hako_abi.h` header generated automatically
- LLVM-generated code can call array/map/string operations
- Same implementation serves Rust plugins AND LLVM code
- Zero performance regression

**Estimated Effort**: 80-120 hours

---

## Detailed Implementation Plan

### Checklist: Phase 1 (Break Circular Dependency)

#### 1.1 Create `hako_abi` crate (2 hours)
```bash
cd crates
cargo new --lib hako_abi
```

**Files to create**:
- [ ] `crates/hako_abi/src/lib.rs`
  - [ ] Re-export all sub-modules
  - [ ] Add crate-level documentation
- [ ] `crates/hako_abi/src/handles.rs`
  - [ ] Define `HakoHandle` type alias
  - [ ] Define `HAKO_INVALID_HANDLE` constant
  - [ ] Document handle semantics
- [ ] `crates/hako_abi/src/types.rs`
  - [ ] Define TLV tag constants (copy from existing code)
  - [ ] Define error code constants
  - [ ] Add doc comments for each constant
- [ ] `crates/hako_abi/src/functions.rs`
  - [ ] Define `ArrayAbi` trait
  - [ ] Define `MapAbi` trait
  - [ ] Define `StringAbi` trait
  - [ ] Add comprehensive doc comments

**Validation**:
```bash
cd crates/hako_abi
cargo build
cargo doc --open  # Check documentation
```

---

#### 1.2 Create `hako_abi_impl` crate (4 hours)
```bash
cd crates
cargo new --lib hako_abi_impl
```

**Files to create**:
- [ ] `crates/hako_abi_impl/Cargo.toml`
  - [ ] Add dependency: `hako_abi = { path = "../hako_abi" }`
  - [ ] Add dependency: `hako_core_array = { path = "../hako_core_array" }`
  - [ ] Add dependency: `hako_core_map = { path = "../hako_core_map" }`
  - [ ] Add dependency: `hako_core_string = { path = "../hako_core_string" }`
  - [ ] Add dependency: `once_cell = "1.19"`
- [ ] `crates/hako_abi_impl/src/lib.rs`
  - [ ] Re-export all implementations
  - [ ] Re-export TLV utilities
- [ ] `crates/hako_abi_impl/src/tlv/mod.rs`
  - [ ] Extract TLV encoding from plugins
  - [ ] Extract TLV decoding from plugins
  - [ ] Add unit tests
- [ ] `crates/hako_abi_impl/src/array_impl.rs`
  - [ ] Implement `ArrayAbi` trait
  - [ ] Add `ArrayRegistry` struct
  - [ ] Add unit tests

**Validation**:
```bash
cd crates/hako_abi_impl
cargo test
cargo build
```

---

#### 1.3 Proof of Concept: Migrate ONE plugin (4 hours)

**Target**: `nyash-array-plugin` (simplest, well-tested)

**Steps**:
- [ ] Add `hako_abi_impl` to `plugins/nyash-array-plugin/Cargo.toml`
- [ ] Refactor `array_invoke_id()` to use shared implementation
- [ ] Delete duplicated TLV encoding/decoding (expect ~100 line reduction)
- [ ] Run existing tests:
  ```bash
  cd plugins/nyash-array-plugin
  cargo test
  ```
- [ ] Run integration tests:
  ```bash
  NYASH_DISABLE_PLUGINS=0 ./target/release/hako apps/tests/array_basic.nyash
  ```

**Success Criteria**:
- All tests pass
- Plugin code reduced by ~100 lines
- No behavior changes (backward compatible)

---

### Checklist: Phase 2 (Unify All Plugins)

#### 2.1 Refactor Core Plugins (8-12 hours each)

**Order** (easiest to hardest):
1. [ ] `nyash-array-plugin` (already done in Phase 1)
2. [ ] `nyash-map-plugin` (similar to array)
3. [ ] `nyash-string-plugin` (similar to array)
4. [ ] `nyash-integer-plugin` (trivial)
5. [ ] `nyash-json-plugin` (medium complexity)
6. [ ] `nyash-filebox-plugin` (medium complexity)
7. [ ] `nyash-net-plugin` (complex, many methods)

**Per-plugin steps**:
- [ ] Add `hako_abi_impl` dependency
- [ ] Identify duplicated logic (TLV, validation, etc.)
- [ ] Replace with shared implementation
- [ ] Run unit tests (`cargo test`)
- [ ] Run integration tests (smoke tests)
- [ ] Measure line reduction
- [ ] Document migration in commit message

---

#### 2.2 Centralize TLV Codec (4 hours)

**Current state**: Every plugin has ~100 lines of TLV encoding/decoding.

**Target state**: All use `hako_abi_impl::tlv`.

**Files to create**:
- [ ] `crates/hako_abi_impl/src/tlv/encode.rs`
  - [ ] `write_tlv_i64()`
  - [ ] `write_tlv_string()`
  - [ ] `write_tlv_handle()`
  - [ ] `write_tlv_host_handle()`
- [ ] `crates/hako_abi_impl/src/tlv/decode.rs`
  - [ ] `read_arg_i64()`
  - [ ] `read_arg_string()`
  - [ ] `read_arg_handle()`
  - [ ] `read_arg_host_handle()`
- [ ] `crates/hako_abi_impl/src/tlv/tests.rs`
  - [ ] Round-trip tests for each type
  - [ ] Error handling tests

---

#### 2.3 Update `nyash_kernel` (4 hours)

**Goal**: Make `nyash_kernel` use shared helpers (not full migration).

**Current code** (156 lines in `array.rs`):
```rust
// Duplicates bounds checking logic
if idx < 0 { return 0; }
if idx >= len { return 0; }
```

**After refactoring**:
```rust
use hako_core_array::safe_get_index;

if let Some(i) = safe_get_index(arr.len(), idx) {
    // Use validated index
}
```

**Files to modify**:
- [ ] `crates/hako_kernel/src/plugin/array.rs`
  - [ ] Use `hako_core_array` helpers
  - [ ] Remove duplicated validation
  - [ ] Expected reduction: ~20 lines
- [ ] `crates/hako_kernel/src/plugin/map.rs`
  - [ ] Use `hako_core_map` helpers
  - [ ] Expected reduction: ~15 lines
- [ ] `crates/hako_kernel/src/plugin/string.rs`
  - [ ] Use `hako_core_string` helpers
  - [ ] Expected reduction: ~10 lines

**Total expected reduction in nyash_kernel**: ~45 lines

---

#### 2.4 Stage-2 Dependencies (For Map.keys/values support)

**Context**: Some Map operations return collections (e.g., `keys()`, `values()`) which require returning HostHandle to Array.

**Requirements**:

1. **Map.keys() / Map.values() Implementation**:
   ```rust
   // crates/hako_abi_impl/src/map_impl.rs
   impl MapAbi for MapRegistry {
       fn map_keys(handle: HakoHandle) -> HakoHandle {
           // Create ArrayBox with all keys
           let keys_array = host_handles::to_handle_arc(Arc::new(ArrayBox::new()));
           // Populate array...
           keys_array as HakoHandle
       }

       fn map_values(handle: HakoHandle) -> HakoHandle {
           // Similar for values
       }
   }
   ```

2. **Build Feature** (Enable cross-crate handle usage):
   ```toml
   # crates/hako_abi_impl/Cargo.toml
   [features]
   host-handle = ["nyash-rust/runtime"]  # Enables access to host_handles module
   ```

3. **Environment Variable** (Runtime flag):
   ```bash
   # Enable Map.keys/values returning HostHandle(Array)
   export HAKO_PLUGIN_MAP_ARRAY_HANDLE=1
   ```

**Migration Notes**:
- **Phase 1**: Can skip this (Map.keys/values not critical)
- **Phase 2**: Implement if needed, or defer to Phase 2.5
- **Test Coverage**: Add tests for `Map.keys()` → `ArrayBox` conversion

**Alternative Approach** (If host_handles is not accessible):
- Return plugin-side Array handle instead
- Add conversion function: `plugin_array_to_host_array()`

---

### Checklist: Phase 3 (C ABI Layer - Future)

#### 3.1 Generate C Header (8 hours)

**Approach**: Use `cbindgen` crate.

**Steps**:
- [ ] Add `cbindgen` to build dependencies
- [ ] Configure `cbindgen.toml`:
  ```toml
  [export]
  prefix = "hako_"
  include = ["ArrayAbi", "MapAbi", "StringAbi"]
  ```
- [ ] Add build script `crates/hako_abi/build.rs`
- [ ] Generate `include/hako_abi.h`
- [ ] Verify header compiles with C compiler:
  ```bash
  gcc -c -I include include/hako_abi.h
  ```

---

#### 3.2 Add C Wrapper Layer (12 hours)

**Create new crate**: `crates/hako_abi_c`

**Purpose**: Thin C wrapper that calls Rust implementation.

```rust
// crates/hako_abi_c/src/lib.rs
use hako_abi::ArrayAbi;
use hako_abi_impl::ArrayRegistry;

#[no_mangle]
pub extern "C" fn hako_array_new() -> u64 {
    ArrayRegistry::array_new()
}

#[no_mangle]
pub extern "C" fn hako_array_get(handle: u64, idx: i64) -> i64 {
    ArrayRegistry::array_get(handle, idx)
}

// ... etc for all ABI functions
```

**Validation**:
- [ ] Build static library: `cargo build --release`
- [ ] Verify symbols exported: `nm -D target/release/libhako_abi_c.a | grep hako_`
- [ ] Write C test program that calls functions
- [ ] Link and run C test

---

#### 3.3 LLVM Integration (16 hours)

**Goal**: Hakorune's LLVM backend generates calls to `hako_*` functions.

**Steps**:
- [ ] Update `src/llvm_py/llvm_builder.py`
  - [ ] Add function declarations for `hako_array_*`, etc.
  - [ ] Generate calls to these functions instead of inline IR
- [ ] Update linker flags to include `libhako_abi_c.a`
- [ ] Test with simple Hakorune program:
  ```hakorune
  static box Main {
      main() {
          local arr = new ArrayBox()
          arr.push(42)
          return arr.get(0)
      }
  }
  ```
- [ ] Verify generated LLVM IR calls `hako_*` functions
- [ ] Verify executable runs correctly

---

## Testing Strategy

### Unit Tests (Per-crate)

#### `hako_abi`
- [ ] Trait method signatures compile
- [ ] Constants have expected values
- [ ] Documentation examples compile

#### `hako_abi_impl`
- [ ] TLV round-trip tests (encode → decode → verify)
- [ ] Array operations (new, get, set, push, len)
- [ ] Map operations (new, get, set, has, size)
- [ ] String operations (new, concat, substring, length)
- [ ] Edge cases (invalid handles, out-of-bounds, etc.)

#### `hako_abi_c` (Phase 3)
- [ ] C test programs for each function
- [ ] Memory leak tests (valgrind)
- [ ] Thread safety tests

---

### Integration Tests

#### Plugin Tests
For each migrated plugin:
- [ ] Run plugin's own test suite (`cargo test`)
- [ ] Run Hakorune integration tests:
  ```bash
  tools/smokes/v2/run.sh --profile quick
  ```
- [ ] Verify no behavior changes (diff outputs)

#### LLVM Tests (Phase 3)
- [ ] Compile simple Hakorune programs to LLVM IR
- [ ] Link with `libhako_abi_c.a`
- [ ] Run and verify outputs
- [ ] Benchmark performance (vs. current implementation)

---

### Regression Tests

**Before migration**:
```bash
# Capture baseline outputs
tools/smokes/v2/run.sh --profile integration > baseline.txt
```

**After each phase**:
```bash
# Verify outputs unchanged
tools/smokes/v2/run.sh --profile integration > after_phase1.txt
diff baseline.txt after_phase1.txt
```

---

## Expected Benefits

### Immediate (Phase 1-2)

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| **Plugin code duplication** | ~1,500 lines | ~300 lines | -80% |
| **TLV codec copies** | 15 copies | 1 copy | -93% |
| **Validation logic copies** | ~500 lines | 1 copy | -99% |
| **Circular dependency** | Yes | No | ✅ Fixed |
| **Single source of truth** | No | Yes | ✅ Achieved |

### Long-term (Phase 3)

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| **C ABI support** | No | Yes | ✅ New capability |
| **LLVM integration** | Inline IR | C function calls | Cleaner IR |
| **Maintainability** | 2+ places | 1 place | ✅ No "禿げます"! |

---

## Architecture Diagrams

### Current Architecture (Problem)
```
┌─────────────────┐         ┌──────────────────┐
│  nyash_kernel   │────────▶│   nyash-rust     │
│  (2,387 lines)  │ imports │ (concrete types) │
└─────────────────┘         └──────────────────┘
        ▲                            ▲
        │ duplicates                 │ duplicates
        │ logic                      │ types
        ▼                            ▼
┌─────────────────────────────────────────────┐
│  plugins (15+)                              │
│  Each has own TLV codec (~100 lines)        │
│  Each reimplements validation (~50 lines)   │
└─────────────────────────────────────────────┘

❌ Circular dependency
❌ Code duplication
❌ No C ABI support
```

### Proposed Architecture (Solution)
```
┌────────────────────────────────────────────┐
│         Layer 1: hako_abi                  │
│  (Pure ABI definitions, ZERO dependencies) │
│  - Traits (ArrayAbi, MapAbi, ...)         │
│  - Constants (TLV tags, error codes)      │
│  - C-compatible types only                │
└────────────────────────────────────────────┘
                    ▲
                    │ implements
                    │
┌────────────────────────────────────────────┐
│      Layer 2: hako_abi_impl                │
│  (Shared implementation, used by all)      │
│  - ArrayRegistry (concrete impl)           │
│  - TLV codec (encode/decode)              │
│  - Uses hako_core_* for validation        │
│  - NO dependency on nyash-rust            │
└────────────────────────────────────────────┘
         ▲                    ▲
         │ uses               │ uses
         │                    │
┌──────────────────┐  ┌───────────────────┐
│ Layer 3a: kernel │  │ Layer 3b: plugins │
│  nyash_kernel    │  │  (15+ plugins)    │
│  - Thin wrapper  │  │  - Thin wrapper   │
│  - Delegates to  │  │  - Delegates to   │
│    impl          │  │    impl           │
└──────────────────┘  └───────────────────┘
         │                    │
         └──────────┬─────────┘
                    ▼
         ┌────────────────────┐
         │  nyash-rust        │
         │  (Box types)       │
         │  NO circular dep!  │
         └────────────────────┘

✅ No circular dependency
✅ Single source of truth
✅ Shared code (めちゃくちゃ使える！)
✅ Future C ABI ready
```

### Future Architecture (Phase 3)
```
┌────────────────────────────────────────────┐
│         Layer 1: hako_abi                  │
│  + Generated C header (hako_abi.h)         │
└────────────────────────────────────────────┘
                    ▲
                    │
┌────────────────────────────────────────────┐
│      Layer 2: hako_abi_impl                │
│  (Same shared implementation)              │
└────────────────────────────────────────────┘
         ▲                    ▲                 ▲
         │                    │                 │
┌────────┴─────┐  ┌───────────┴──────┐  ┌──────┴────────┐
│ Rust plugins │  │  nyash_kernel    │  │ hako_abi_c    │
└──────────────┘  └──────────────────┘  │ (C wrapper)   │
                                        └───────────────┘
                                               ▲
                                               │ calls
                                               │
                                        ┌──────┴────────┐
                                        │ LLVM-generated│
                                        │     code      │
                                        │ (C ABI calls) │
                                        └───────────────┘

✅ One implementation
✅ Three consumers (Rust, Kernel, LLVM)
✅ C ABI compatibility
```

---

## Risk Analysis & Mitigation

### Risk 1: Breaking Plugin API
**Probability**: Medium
**Impact**: High (plugins stop working)

**Mitigation**:
- Migrate plugins one by one
- Keep old code path until all plugins migrated
- Extensive regression testing
- Feature flag to enable/disable new implementation

---

### Risk 2: Performance Regression
**Probability**: Low
**Impact**: Medium

**Mitigation**:
- Benchmark before/after each phase
- Profile hot paths (TLV encoding/decoding)
- Use `#[inline]` for critical functions
- Accept small regression for maintainability gains

---

### Risk 3: Incomplete Migration (abandoned halfway)
**Probability**: Medium
**Impact**: High (mixed codebase, worse than before)

**Mitigation**:
- Start with Phase 1 (small, self-contained)
- Each phase delivers value independently
- Clear success criteria per phase
- Document fallback plan in each phase

---

## Alternative Approaches Considered

### Alternative 1: Keep Current Architecture, Use Macros
**Idea**: Generate plugin code with macros.

**Pros**:
- No new crates
- Minimal disruption

**Cons**:
- Doesn't fix circular dependency
- Macros hard to debug
- Doesn't enable C ABI

**Decision**: ❌ Rejected

---

### Alternative 2: Merge nyash_kernel into nyash-rust
**Idea**: Move all ABI functions into main crate.

**Pros**:
- No circular dependency
- Simpler crate structure

**Cons**:
- Bloats main crate with ABI code
- Plugins still can't share code
- Doesn't enable C ABI

**Decision**: ❌ Rejected

---

### Alternative 3: Plugin SDK Crate
**Idea**: Create `hako_plugin_sdk` with utilities.

**Pros**:
- Clear separation
- Plugin-focused

**Cons**:
- Doesn't help nyash_kernel
- Doesn't enable C ABI
- Still duplicates logic

**Decision**: ❌ Rejected (but SDK could be Layer 4 in future)

---

## Success Metrics

### Phase 1 Success Criteria
- [ ] `hako_abi` crate compiles with ZERO dependencies
- [ ] `hako_abi_impl` compiles without `nyash-rust` dependency
- [ ] ONE plugin successfully migrated
- [ ] All existing tests pass
- [ ] No performance regression (< 5%)

### Phase 2 Success Criteria
- [ ] ALL plugins use `hako_abi_impl`
- [ ] Total line reduction: 500-800 lines
- [ ] TLV codec centralized (1 copy, not 15)
- [ ] All smoke tests pass
- [ ] No behavior changes

### Phase 3 Success Criteria
- [ ] C header generated successfully
- [ ] LLVM-generated code calls C ABI functions
- [ ] Same implementation serves all consumers
- [ ] Performance within 10% of current implementation

---

## Timeline Estimate

| Phase | Duration | Effort (hours) | Blocker |
|-------|----------|----------------|---------|
| **Phase 1** | 1-2 days | 8-12 | None |
| **Phase 2** | 1-2 weeks | 40-80 | Phase 1 complete |
| **Phase 3** | 2-4 weeks | 80-120 | Phase 2 complete |
| **Total** | 4-7 weeks | 128-212 | - |

**Recommendation**: Start with Phase 1 immediately (high value, low risk).

---

## References

### Inspiration from Industry
- **wasmtime**: Opaque handle types + C API layer
- **V8**: Isolate + Handle + C++ API
- **Python C API**: PyObject* handles + reference counting
- **Rust FFI Guide**: https://doc.rust-lang.org/nomicon/ffi.html

### Hakorune Documentation
- `docs/reference/plugin-system/` - Plugin architecture
- `src/runtime/host_handles.rs` - Current handle system
- `crates/hako_core_*/` - Shared validation logic

---

## Conclusion

This 3-layer architecture achieves ALL goals:

1. ✅ **Code sharing**: Plugins and core use SAME implementation
2. ✅ **No circular dependency**: `hako_abi_impl` independent of `nyash-rust`
3. ✅ **Future C ABI**: Design supports LLVM-generated code
4. ✅ **Single source of truth**: All ABI logic in ONE place
5. ✅ **めちゃくちゃ使える！**: No more "2カ所管理で禿げます"!

**Recommended Next Steps**:
1. Review this design (30 min)
2. Start Phase 1.1: Create `hako_abi` crate (2 hours)
3. Validate approach with proof-of-concept
4. Decide: Continue to Phase 2 or adjust design

**Questions for Review**:
- Does this architecture satisfy "めちゃくちゃ使いたい" vision?
- Are there concerns about migration risk?
- Should we proceed with Phase 1?

---

**End of Design Document**
