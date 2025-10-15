# Hakorune ABI Quick Start Guide

**For**: Developers ready to implement the 3-layer ABI architecture
**Reading time**: 10 minutes
**Implementation time**: 8-12 hours (Phase 1 only)

---

## TL;DR

We're creating 2 new crates to eliminate code duplication:

```
hako_abi         → Pure ABI definitions (traits, no deps)
hako_abi_impl    → Shared implementation (used by kernel + plugins)
```

**Result**: 500-800 lines deleted, no more "2カ所管理で禿げます"!

---

### Build Requirements (MUST configure first!)

**⚠️ Critical**: Before starting Phase 1, ensure your build is configured correctly.

#### 1. Linker Flags (Required for host API exports)

Add to `.cargo/config.toml`:
```toml
[target.x86_64-unknown-linux-gnu]
rustflags = ["-Clink-arg=-rdynamic"]

[target.x86_64-apple-darwin]
rustflags = ["-Clink-arg=-rdynamic"]

[target.aarch64-apple-darwin]
rustflags = ["-Clink-arg=-rdynamic"]
```

**Why**: The `-rdynamic` flag ensures that host API functions are exported as dynamic symbols, allowing plugins to find them via `dlsym()` at runtime.

#### 2. Anchor Functions (Prevent LTO dead code elimination)

Create `crates/hako_kernel/src/plugin/anchors.rs`:
```rust
//! Host API Symbol Anchors - Force-link host C ABI symbols

use super::array::*;
use super::map::*;
use super::string::*;

/// Wrapper to make function pointer array Sync-safe
pub struct FnPtrArray(pub &'static [*const ()]);
unsafe impl Sync for FnPtrArray {}

/// Host API function pointers - prevents dead code elimination
#[used]
#[no_mangle]
pub static NYASH_HOST_API_ANCHORS: FnPtrArray = FnPtrArray(&[
    nyash_array_new_h as *const (),
    nyash_array_get_h as *const (),
    nyash_array_set_h as *const (),
    nyash_array_push_h as *const (),
    nyash_array_length_h as *const (),
    nyash_map_size_h as *const (),
    nyash_map_get_h as *const (),
    nyash_map_set_h as *const (),
    // Add all host API functions here
]);
```

**Why**: Even with `-rdynamic`, LTO (Link Time Optimization) may remove functions that appear unused. The `#[used]` attribute on a static reference forces the linker to retain these symbols.

#### 3. Verification (After building)

Verify host API symbols are exported:
```bash
cargo build --release
nm target/release/hako | grep nyash_array_new_h
# Should output: 0000000000240540 T nyash_array_new_h
```

If the symbol is missing, check:
1. `.cargo/config.toml` has `-rdynamic` for your platform
2. `anchors.rs` is included in your crate
3. LTO is not too aggressive (add `lto = "thin"` in `Cargo.toml` if needed)

---

## Phase 1: The First 12 Hours

### Hour 1-2: Create `hako_abi`

```bash
cd /home/tomoaki/git/hakorune-selfhost/crates
cargo new --lib hako_abi
cd hako_abi
```

**File 1**: `src/handles.rs`
```rust
//! Handle type definitions for Hakorune ABI

/// Opaque handle to Hakorune objects (u64 internally)
pub type HakoHandle = u64;

/// Invalid/null handle constant
pub const HAKO_INVALID_HANDLE: HakoHandle = 0;
```

**File 2**: `src/types.rs`
```rust
//! TLV type tags and error codes

// TLV tags (synchronized with all implementations)
pub const TLV_TAG_I64: u8 = 3;
pub const TLV_TAG_STRING: u8 = 6;
pub const TLV_TAG_PLUGIN_HANDLE: u8 = 8;
pub const TLV_TAG_HOST_HANDLE: u8 = 9;

// Error codes
pub const HAKO_SUCCESS: i32 = 0;
pub const HAKO_E_SHORT_BUFFER: i32 = -1;
pub const HAKO_E_INVALID_ARGS: i32 = -2;
pub const HAKO_E_INVALID_HANDLE: i32 = -8;
```

**File 3**: `src/array.rs`
```rust
//! Array ABI contract

use super::handles::HakoHandle;

/// Array operations ABI
pub trait ArrayAbi {
    /// Create new array
    fn array_new() -> HakoHandle;

    /// Get element at index (returns i64 or 0 if not found)
    fn array_get(handle: HakoHandle, idx: i64) -> i64;

    /// Set element at index (returns 0 on success)
    fn array_set(handle: HakoHandle, idx: i64, val: i64) -> i64;

    /// Push element (returns new length)
    fn array_push(handle: HakoHandle, val: i64) -> i64;

    /// Get array length
    fn array_len(handle: HakoHandle) -> i64;
}
```

**File 4**: `src/lib.rs`
```rust
//! Hakorune ABI Definitions
//!
//! Pure trait-based ABI contracts with ZERO dependencies.
//! Used by both core and plugins.

pub mod handles;
pub mod types;
pub mod array;

// Re-exports for convenience
pub use handles::{HakoHandle, HAKO_INVALID_HANDLE};
pub use types::*;
pub use array::ArrayAbi;
```

**Test it**:
```bash
cargo build
cargo doc --open  # Verify docs look good
```

**Expected**: Clean compile, zero warnings, nice docs.

---

### Hour 3-6: Create `hako_abi_impl`

```bash
cd /home/tomoaki/git/hakorune-selfhost/crates
cargo new --lib hako_abi_impl
cd hako_abi_impl
```

**File 1**: `Cargo.toml`
```toml
[package]
name = "hako_abi_impl"
version = "0.1.0"
edition = "2021"

[dependencies]
hako_abi = { path = "../hako_abi" }
hako_core_array = { path = "../hako_core_array" }
once_cell = "1.19"
```

**File 2**: `src/tlv.rs`
```rust
//! TLV encoding/decoding utilities (shared by all)

use hako_abi::{TLV_TAG_I64, HAKO_SUCCESS, HAKO_E_SHORT_BUFFER, HAKO_E_INVALID_ARGS};

/// Read i64 from TLV-encoded args at position n
pub fn read_arg_i64(args: *const u8, args_len: usize, n: usize) -> Option<i64> {
    if args.is_null() || args_len < 4 {
        return None;
    }
    let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
    let mut off = 4usize; // Skip header (version + argc)

    for i in 0..=n {
        if buf.len() < off + 4 {
            return None;
        }
        let tag = buf[off];
        let size = u16::from_le_bytes([buf[off + 2], buf[off + 3]]) as usize;

        if buf.len() < off + 4 + size {
            return None;
        }

        if i == n {
            if tag != TLV_TAG_I64 || size != 8 {
                return None;
            }
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&buf[off + 4..off + 12]);
            return Some(i64::from_le_bytes(bytes));
        }

        off += 4 + size;
    }

    None
}

/// Write i64 to TLV-encoded result buffer
pub fn write_tlv_i64(val: i64, result: *mut u8, result_len: *mut usize) -> i32 {
    if result_len.is_null() {
        return HAKO_E_INVALID_ARGS;
    }

    let mut buf = Vec::with_capacity(16);
    buf.extend_from_slice(&1u16.to_le_bytes()); // version
    buf.extend_from_slice(&1u16.to_le_bytes()); // argc
    buf.push(TLV_TAG_I64);
    buf.push(0); // reserved
    buf.extend_from_slice(&8u16.to_le_bytes()); // size
    buf.extend_from_slice(&val.to_le_bytes());

    unsafe {
        let needed = buf.len();
        if result.is_null() || *result_len < needed {
            *result_len = needed;
            return HAKO_E_SHORT_BUFFER;
        }
        std::ptr::copy_nonoverlapping(buf.as_ptr(), result, needed);
        *result_len = needed;
    }

    HAKO_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i64_roundtrip() {
        // Encode
        let mut buf = [0u8; 256];
        let mut len = buf.len();
        let result = write_tlv_i64(42, buf.as_mut_ptr(), &mut len);
        assert_eq!(result, HAKO_SUCCESS);

        // Decode
        let val = read_arg_i64(buf.as_ptr(), len, 0);
        assert_eq!(val, Some(42));
    }
}
```

**File 3**: `src/array_impl.rs`
```rust
//! Array ABI implementation (shared by all consumers)

use hako_abi::{ArrayAbi, HakoHandle, HAKO_INVALID_HANDLE};
use hako_core_array::{classify_set_index, SetIndex};
use std::collections::HashMap;
use std::sync::{Mutex, atomic::{AtomicU64, Ordering}};

/// Array element value (plugin-side, no NyashBox dependency)
#[derive(Clone, Debug)]
pub enum ArrayValue {
    I64(i64),
    // TODO: Add String, Handle variants
}

/// Single array instance
struct ArrayInstance {
    data: Vec<ArrayValue>,
}

/// Thread-safe registry of all array instances
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

    fn alloc(&self) -> HakoHandle {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let mut map = self.instances.lock().unwrap();
        map.insert(id, ArrayInstance { data: Vec::new() });
        id
    }

    fn with_instance<F, R>(&self, handle: HakoHandle, f: F) -> Option<R>
    where
        F: FnOnce(&ArrayInstance) -> R,
    {
        let map = self.instances.lock().unwrap();
        map.get(&handle).map(f)
    }

    fn with_instance_mut<F, R>(&self, handle: HakoHandle, f: F) -> Option<R>
    where
        F: FnOnce(&mut ArrayInstance) -> R,
    {
        let mut map = self.instances.lock().unwrap();
        map.get_mut(&handle).map(f)
    }
}

impl ArrayAbi for ArrayRegistry {
    fn array_new() -> HakoHandle {
        REGISTRY.alloc()
    }

    fn array_get(handle: HakoHandle, idx: i64) -> i64 {
        REGISTRY
            .with_instance(handle, |inst| {
                if let Some(i) = hako_core_array::safe_get_index(inst.data.len(), idx) {
                    match &inst.data[i] {
                        ArrayValue::I64(v) => *v,
                    }
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    fn array_set(handle: HakoHandle, idx: i64, val: i64) -> i64 {
        REGISTRY
            .with_instance_mut(handle, |inst| {
                match classify_set_index(inst.data.len(), idx) {
                    SetIndex::Replace(i) => {
                        inst.data[i] = ArrayValue::I64(val);
                        0
                    }
                    SetIndex::Append => {
                        inst.data.push(ArrayValue::I64(val));
                        0
                    }
                    SetIndex::Oob => -1,
                }
            })
            .unwrap_or(-1)
    }

    fn array_push(handle: HakoHandle, val: i64) -> i64 {
        REGISTRY
            .with_instance_mut(handle, |inst| {
                inst.data.push(ArrayValue::I64(val));
                inst.data.len() as i64
            })
            .unwrap_or(0)
    }

    fn array_len(handle: HakoHandle) -> i64 {
        REGISTRY
            .with_instance(handle, |inst| hako_core_array::length(inst.data.len()))
            .unwrap_or(0)
    }
}

static REGISTRY: once_cell::sync::Lazy<ArrayRegistry> =
    once_cell::sync::Lazy::new(ArrayRegistry::new);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_basic() {
        let h = ArrayRegistry::array_new();
        assert_ne!(h, HAKO_INVALID_HANDLE);

        // Push
        let len = ArrayRegistry::array_push(h, 42);
        assert_eq!(len, 1);

        // Get
        let val = ArrayRegistry::array_get(h, 0);
        assert_eq!(val, 42);

        // Len
        let len = ArrayRegistry::array_len(h);
        assert_eq!(len, 1);
    }

    #[test]
    fn test_array_set() {
        let h = ArrayRegistry::array_new();
        ArrayRegistry::array_push(h, 10);

        // Replace
        let result = ArrayRegistry::array_set(h, 0, 20);
        assert_eq!(result, 0);
        assert_eq!(ArrayRegistry::array_get(h, 0), 20);

        // Append
        let result = ArrayRegistry::array_set(h, 1, 30);
        assert_eq!(result, 0);
        assert_eq!(ArrayRegistry::array_len(h), 2);

        // Out of bounds
        let result = ArrayRegistry::array_set(h, 10, 40);
        assert_eq!(result, -1);
    }
}
```

**File 4**: `src/lib.rs`
```rust
//! Hakorune ABI Implementation
//!
//! Shared implementation used by both nyash_kernel and plugins.
//! NO dependency on nyash-rust to avoid circular dependency.

pub mod tlv;
pub mod array_impl;

// Re-exports
pub use hako_abi;
pub use array_impl::ArrayRegistry;
```

**Test it**:
```bash
cargo test
cargo build
```

**Expected**: All tests pass, including roundtrip and bounds checking.

---

### Hour 7-10: Migrate ONE Plugin (Proof of Concept)

**Target**: `nyash-array-plugin`

**Step 1**: Add dependency
```toml
# plugins/nyash-array-plugin/Cargo.toml
[dependencies]
hako_abi_impl = { path = "../../crates/hako_abi_impl" }
```

**Step 2**: Refactor `METHOD_GET` (example)

**BEFORE** (50 lines):
```rust
METHOD_GET => {
    let idx = match read_arg_i64(args, args_len, 0) {
        Some(v) => v,
        None => return NYB_E_INVALID_ARGS,
    };
    if idx < 0 {
        return NYB_E_INVALID_ARGS;
    }
    if let Ok(map) = INSTANCES.lock() {
        if let Some(inst) = map.get(&instance_id) {
            let i = idx as usize;
            if i >= inst.data.len() {
                return NYB_E_INVALID_ARGS;
            }
            return write_tlv_value(&inst.data[i], result, result_len);
        } else {
            return NYB_E_INVALID_HANDLE;
        }
    } else {
        return NYB_E_PLUGIN_ERROR;
    }
}
```

**AFTER** (10 lines):
```rust
METHOD_GET => {
    let idx = hako_abi_impl::tlv::read_arg_i64(args, args_len, 0)
        .ok_or(NYB_E_INVALID_ARGS)?;

    let val = hako_abi_impl::ArrayRegistry::array_get(instance_id as u64, idx);

    hako_abi_impl::tlv::write_tlv_i64(val, result, result_len)
}
```

**Savings**: 40 lines → 10 lines (75% reduction!)

**Step 3**: Delete duplicated TLV functions

Delete these from plugin:
- `read_arg_i64()` (30 lines)
- `write_tlv_i64()` (25 lines)
- `read_arg_string()` (30 lines)
- etc.

**Total savings per plugin**: ~100-150 lines

**Step 4**: Test
```bash
cd plugins/nyash-array-plugin
cargo test
cargo build

# Integration test
cd ../..
NYASH_DISABLE_PLUGINS=0 ./target/release/hako apps/tests/array_basic.nyash
```

---

### Hour 11-12: Documentation and Review

**Create migration checklist**:
```markdown
## Phase 1 Completion Checklist

- [x] hako_abi crate created (ZERO dependencies)
- [x] hako_abi_impl crate created
- [x] TLV codec centralized
- [x] Array implementation shared
- [x] ONE plugin migrated (nyash-array-plugin)
- [x] All tests pass
- [ ] Code review
- [ ] Measure line reduction (expect ~100 lines)
- [ ] Performance benchmark (expect < 5% regression)
```

**Write commit message**:
```
feat(abi): Phase 1 - Create shared ABI layer

BREAKING: Refactors nyash-array-plugin to use shared implementation.

Changes:
- New crate: hako_abi (pure ABI definitions)
- New crate: hako_abi_impl (shared implementation)
- Migrated: nyash-array-plugin (~100 line reduction)
- Centralized: TLV encoding/decoding
- Tests: All passing (unit + integration)

Benefits:
- Eliminates code duplication
- Single source of truth for ABI logic
- Prepares for future C ABI migration

Next: Phase 2 - Migrate remaining plugins
```

---

## Quick Reference: Common Tasks

### Add new ABI function

**1. Define in `hako_abi`**:
```rust
// crates/hako_abi/src/array.rs
pub trait ArrayAbi {
    fn array_slice(handle: HakoHandle, start: i64, end: i64) -> HakoHandle;
}
```

**2. Implement in `hako_abi_impl`**:
```rust
// crates/hako_abi_impl/src/array_impl.rs
impl ArrayAbi for ArrayRegistry {
    fn array_slice(handle: HakoHandle, start: i64, end: i64) -> HakoHandle {
        // Implementation here
    }
}
```

**3. Use in plugin**:
```rust
let slice_handle = hako_abi_impl::ArrayRegistry::array_slice(handle, 0, 5);
```

---

### Add new TLV type

**1. Define tag in `hako_abi`**:
```rust
// crates/hako_abi/src/types.rs
pub const TLV_TAG_FLOAT: u8 = 10;
```

**2. Add codec in `hako_abi_impl`**:
```rust
// crates/hako_abi_impl/src/tlv.rs
pub fn read_arg_f64(args: *const u8, args_len: usize, n: usize) -> Option<f64> {
    // Implementation
}

pub fn write_tlv_f64(val: f64, result: *mut u8, result_len: *mut usize) -> i32 {
    // Implementation
}
```

---

### Debug TLV encoding issues

```rust
// Temporary debug code
let buf = unsafe { std::slice::from_raw_parts(args, args_len) };
eprintln!("TLV debug: len={} bytes={:?}", args_len, &buf[..args_len.min(32)]);

// Decode and print
if let Some(val) = hako_abi_impl::tlv::read_arg_i64(args, args_len, 0) {
    eprintln!("Decoded arg0: {}", val);
} else {
    eprintln!("Failed to decode arg0");
}
```

---

## Troubleshooting

### Issue: Circular dependency error
```
error[E0391]: cycle detected when computing layout of `...`
```

**Solution**: Make sure `hako_abi_impl` does NOT depend on `nyash-rust`.
Check `Cargo.toml`:
```toml
# ❌ BAD
[dependencies]
nyash-rust = { path = "../../" }

# ✅ GOOD
[dependencies]
hako_abi = { path = "../hako_abi" }
hako_core_array = { path = "../hako_core_array" }
```

---

### Issue: Tests fail after migration
```
test array_get_basic ... FAILED
```

**Debug steps**:
1. Check TLV encoding matches old format
2. Add debug prints in `hako_abi_impl`
3. Compare before/after with hex dumps
4. Verify bounds checking logic unchanged

---

### Issue: Plugin not loading
```
Error: Plugin symbol not found
```

**Solution**: Make sure `#[no_mangle]` is still on plugin entry points:
```rust
#[no_mangle]
pub extern "C" fn nyash_plugin_invoke(...) -> i32 {
    // This must stay!
}
```

---

## Next Steps After Phase 1

### Phase 2: Migrate All Plugins (40-80 hours)

**Order** (easiest first):
1. nyash-array-plugin ✅ (done in Phase 1)
2. nyash-map-plugin (similar to array)
3. nyash-string-plugin (similar to array)
4. nyash-integer-plugin (trivial)
5. ... (remaining 11 plugins)

**Per-plugin checklist**:
- [ ] Add `hako_abi_impl` dependency
- [ ] Replace TLV codec with shared implementation
- [ ] Replace validation logic with `hako_core_*` helpers
- [ ] Delete duplicated code
- [ ] Run tests
- [ ] Measure line reduction

---

### Phase 3: C ABI Layer (80-120 hours)

**Future work** (after all plugins migrated):
- Generate C header with `cbindgen`
- Create `hako_abi_c` wrapper crate
- Integrate with LLVM backend
- Test from LLVM-generated code

---

## Success Metrics (Phase 1)

| Metric | Target | Actual |
|--------|--------|--------|
| Time spent | 8-12 hours | ______ |
| Lines deleted | ~100 | ______ |
| Tests passing | 100% | ______ |
| Performance regression | < 5% | ______ |

---

## Questions?

**Q**: Why separate `hako_abi` and `hako_abi_impl`?
**A**: `hako_abi` has ZERO dependencies, making it safe for any crate to import. Future C header generation needs this.

**Q**: Won't this make builds slower (more crates)?
**A**: Minimal impact. Incremental builds only recompile changed crates.

**Q**: What if Phase 2/3 get abandoned?
**A**: Phase 1 alone delivers value (shared TLV codec). Each phase is independently useful.

**Q**: Can I keep old code path during migration?
**A**: Yes! Use feature flags:
```rust
#[cfg(feature = "new_abi")]
use hako_abi_impl::ArrayRegistry;

#[cfg(not(feature = "new_abi"))]
use old_implementation::ArrayRegistry;
```

---

**Ready to start?** → Begin with Hour 1-2 (create `hako_abi` crate)!
