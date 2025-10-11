# Hako ABI Architecture Analysis: Circular Dependency Problem

**Date**: 2025-10-11
**Status**: Architecture Analysis
**Problem**: Circular dependency between `nyash_kernel` and `nyash-rust`

---

## Executive Summary

The `nyash_kernel` crate was created to enable **shared ABI code** usable in BOTH plugins AND core kernel. However, it currently has a **circular dependency** with `nyash-rust`, which violates Rust's dependency rules and prevents the intended architecture from working.

**Current Problem**:
```
nyash_kernel → nyash-rust (for runtime types)
nyash-rust → nyash_kernel (intended, for ABI functions)
```

**Root Cause**: `nyash_kernel` imports concrete runtime types from `nyash-rust`, creating the cycle.

---

## Current Architecture Diagram

```
┌─────────────────────────────────────────────────┐
│                  nyash-rust                     │
│  (Core Runtime + Box Trait + Plugin System)     │
│                                                  │
│  ├─ box_trait::{NyashBox, StringBox, etc}       │
│  ├─ runtime::host_handles                       │
│  ├─ runtime::plugin_loader_v2                   │
│  ├─ runtime::plugin_ffi_common                  │
│  └─ boxes::{ArrayBox, MapBox, FloatBox, etc}    │
└─────────────────────────────────────────────────┘
                         ▲
                         │ depends on (CIRCULAR!)
                         │
┌─────────────────────────────────────────────────┐
│                 nyash_kernel                    │
│     (Shared ABI Layer - Plugin + Core)          │
│                                                  │
│  ├─ plugin/invoke.rs (plugin invoke shims)      │
│  ├─ plugin/birth.rs (birth shims)               │
│  ├─ plugin/string.rs, console.rs, etc           │
│  └─ lib.rs (AOT/JIT extern functions)           │
│                                                  │
│  Exports:                                        │
│  - nyash_plugin_invoke3_i64                     │
│  - nyash.string.len_h                           │
│  - nyash.box.birth_h                            │
│  - etc. (50+ ABI functions)                     │
└─────────────────────────────────────────────────┘
                         ▲
                         │ used by
                         │
┌─────────────────────────────────────────────────┐
│                   Plugins                       │
│                                                  │
│  ├─ nyash-console-plugin                        │
│  ├─ nyash-string-plugin                         │
│  ├─ nyash-math-plugin                           │
│  └─ etc.                                        │
│                                                  │
│  Note: Plugins currently DO NOT use             │
│        nyash_kernel functions (they have        │
│        their own invoke implementations)        │
└─────────────────────────────────────────────────┘
```

---

## What nyash_kernel Imports from nyash-rust

### Critical Dependencies (causing circular reference):

1. **Runtime Types**:
   - `nyash_rust::runtime::host_handles` (handle registry)
   - `nyash_rust::runtime::plugin_loader_v2::PluginBoxV2`
   - `nyash_rust::runtime::get_global_plugin_host()`
   - `nyash_rust::runtime::init_global_plugin_host()`
   - `nyash_rust::runtime::plugin_ffi_common` (TLV encoding/decoding)
   - `nyash_rust::runtime::global_hooks` (GC hooks)

2. **Box Trait System**:
   - `nyash_rust::box_trait::{NyashBox, StringBox, IntegerBox, BoolBox, VoidBox}`
   - `nyash_rust::boxes::{ArrayBox, MapBox, FloatBox, BufferBox}`

3. **Plugin Registry**:
   - `nyash_rust::runtime::box_registry::get_global_registry()`

### Why These Are Problematic:

These are **concrete runtime types** from the core `nyash-rust` crate. This creates a hard dependency that makes `nyash_kernel` unable to be a truly shared layer that `nyash-rust` can depend on.

---

## Current Plugin Usage

**Important Discovery**: Plugins currently **DO NOT** use `nyash_kernel` functions!

### What Plugins Actually Do:

1. **Own ABI Implementation**:
   ```rust
   // Each plugin implements its own:
   extern "C" fn nyash_plugin_invoke(
       type_id: u32,
       method_id: u32,
       instance_id: u32,
       args: *const u8,
       args_len: usize,
       result: *mut u8,
       result_len: *mut usize,
   ) -> i32 { ... }
   ```

2. **TypeBox FFI Struct** (newer approach):
   ```rust
   #[no_mangle]
   pub static nyash_typebox_ConsoleBox: NyashTypeBoxFfi = NyashTypeBoxFfi {
       abi_tag: 0x54594258, // 'TYBX'
       version: 1,
       name: b"ConsoleBox\0".as_ptr(),
       resolve: Some(console_resolve),
       invoke_id: Some(console_invoke_id),
       ...
   };
   ```

3. **No External Dependencies**: Plugins use only:
   - Standard Rust types
   - TLV encoding (duplicated in each plugin)
   - Own instance management

---

## Architectural Options

### Option 1: **Trait-Based Abstraction** (Recommended)

**Concept**: Extract all dependencies into traits, move concrete implementations to `nyash-rust`.

```
┌─────────────────────────────────┐
│      nyash_kernel_traits        │
│  (Pure trait definitions only)  │
│                                  │
│  pub trait HandleRegistry {     │
│    fn get(&self, h: u64) -> ... │
│    fn to_handle(...) -> ...     │
│  }                              │
│                                  │
│  pub trait BoxTrait {           │
│    fn as_any(&self) -> ...      │
│    fn to_string_box(...) -> ... │
│  }                              │
└─────────────────────────────────┘
           ▲                ▲
           │                │
           │                │
┌──────────┴──────┐  ┌──────┴───────────────┐
│  nyash_kernel   │  │    nyash-rust        │
│  (uses traits)  │  │ (implements traits)  │
└─────────────────┘  └──────────────────────┘
```

**Pros**:
- ✅ Clean separation of concerns
- ✅ No circular dependencies
- ✅ Compile-time trait resolution (zero overhead)
- ✅ Standard Rust pattern

**Cons**:
- ⚠️ Requires significant refactoring
- ⚠️ More crates to manage

**Implementation Steps**:
1. Create `nyash_kernel_traits` crate
2. Define `HandleRegistry`, `BoxTrait`, `PluginHost`, `TlvCodec` traits
3. Move `nyash_kernel` to use only traits
4. Implement traits in `nyash-rust`
5. Provide trait objects at runtime

---

### Option 2: **C-Compatible ABI Layer** (Minimal Coupling)

**Concept**: Define ABI using only C-compatible types, no Rust runtime dependencies.

```
┌─────────────────────────────────┐
│       nyash_abi_ffi             │
│  (C-compatible types only)      │
│                                  │
│  #[repr(C)]                     │
│  struct HandleId(u64);          │
│                                  │
│  extern "C" fn                  │
│  nyash_handle_get(              │
│    h: HandleId                  │
│  ) -> *const u8;                │
└─────────────────────────────────┘
           ▲                ▲
           │                │
           │                │
┌──────────┴──────┐  ┌──────┴───────────────┐
│  nyash_kernel   │  │    nyash-rust        │
│  (C ABI layer)  │  │  (Rust runtime)      │
└─────────────────┘  └──────────────────────┘
```

**Pros**:
- ✅ Maximum decoupling
- ✅ Works across language boundaries
- ✅ Stable ABI (no Rust runtime coupling)

**Cons**:
- ⚠️ Loses Rust type safety
- ⚠️ Manual memory management
- ⚠️ More verbose code

---

### Option 3: **Global Registry Pattern** (Runtime Injection)

**Concept**: Use global state for runtime injection, similar to current `get_global_plugin_host()`.

```rust
// nyash_kernel:
static RUNTIME: OnceLock<Box<dyn RuntimeBridge>> = OnceLock::new();

pub fn init_runtime(rt: Box<dyn RuntimeBridge>) {
    RUNTIME.set(rt).ok();
}

pub fn get_handle(h: u64) -> Option<...> {
    RUNTIME.get()?.get_handle(h)
}

// nyash-rust:
fn main() {
    nyash_kernel::init_runtime(Box::new(MyRuntime));
}
```

**Pros**:
- ✅ Minimal code changes
- ✅ Runtime flexibility
- ✅ No circular deps

**Cons**:
- ⚠️ Global state (testing harder)
- ⚠️ Runtime initialization required
- ⚠️ Panic if not initialized

---

### Option 4: **Separate ABI Crate with Re-exports** (Quick Fix)

**Concept**: Move ABI functions to separate crate, re-export from both.

```
┌─────────────────────────────────┐
│      nyash_abi_core             │
│  (ABI implementations)          │
│                                  │
│  Depends on: nyash-rust         │
│  Exports: all ABI functions     │
└─────────────────────────────────┘
           ▲                ▲
           │                │
           │                │
┌──────────┴──────┐  ┌──────┴───────────────┐
│  nyash_kernel   │  │    nyash-rust        │
│  (re-exports)   │  │  (re-exports)        │
└─────────────────┘  └──────────────────────┘
```

**Pros**:
- ✅ Minimal refactoring
- ✅ Preserves existing API
- ✅ Quick to implement

**Cons**:
- ⚠️ Doesn't solve real problem (just hides it)
- ⚠️ Confusion about true source
- ⚠️ Not truly shared code

---

### Option 5: **Plugin-Only ABI Crate** (Accept Reality)

**Concept**: Accept that `nyash_kernel` is for plugins only, core uses `nyash-rust` directly.

```
┌─────────────────────────────────┐
│       nyash-rust                │
│  (Core runtime - used by AOT)   │
└─────────────────────────────────┘
                    │
                    │ provides runtime
                    ▼
┌─────────────────────────────────┐
│    nyash_plugin_abi             │
│  (Plugin ABI - uses runtime)    │
│                                  │
│  - Links to nyash-rust          │
│  - Provides plugin helpers      │
└─────────────────────────────────┘
                    ▲
                    │ used by
                    │
┌─────────────────────────────────┐
│         Plugins                 │
└─────────────────────────────────┘
```

**Pros**:
- ✅ Accepts current reality
- ✅ No architecture changes
- ✅ Clear separation of concerns

**Cons**:
- ⚠️ Original goal abandoned (shared code)
- ⚠️ AOT still links nyash-rust directly
- ⚠️ Plugins don't actually use it yet

---

## Recommendation

**Recommended Approach**: **Option 1 (Trait-Based Abstraction)** + **Option 5 (Accept Plugin-Only Reality)**

### Phase 1: Accept Current Reality (Immediate)

1. **Rename** `nyash_kernel` → `nyash_plugin_abi`
2. **Document** that it's for plugin helpers, not core runtime
3. **Keep** dependency on `nyash-rust` (it's okay for plugin layer)
4. **Core/AOT** continues using `nyash-rust` directly

### Phase 2: Trait-Based Future (Long-term)

If true sharing is needed later:

1. Create `nyash_runtime_traits` crate (trait definitions)
2. Both `nyash-rust` and `nyash_plugin_abi` depend on traits
3. Runtime provides trait implementations via dependency injection
4. Plugins can use either:
   - `nyash_plugin_abi` (convenience helpers)
   - Direct trait implementations (minimal coupling)

### Why This Approach?

1. **Pragmatic**: Solves immediate circular dependency
2. **Honest**: Reflects what code actually does
3. **Future-proof**: Leaves door open for trait-based sharing
4. **Minimal disruption**: Rename + documentation, not full rewrite

---

## Implementation Checklist

### Immediate (Phase 1):

- [ ] Rename `crates/nyash_kernel` → `crates/nyash_plugin_abi`
- [ ] Update `Cargo.toml` dependencies
- [ ] Update documentation to clarify purpose
- [ ] Add comment: "This is NOT used by core runtime, only plugin helpers"
- [ ] Test that plugins still build

### Future (Phase 2, if needed):

- [ ] Create `nyash_runtime_traits` crate
- [ ] Define core traits (`HandleRegistry`, `BoxTrait`, etc.)
- [ ] Implement traits in `nyash-rust`
- [ ] Update `nyash_plugin_abi` to use traits
- [ ] Provide trait objects via global registry
- [ ] Test cross-crate trait resolution

---

## Key Insights

1. **Plugins don't use nyash_kernel**: They have own implementations
2. **Circular dep is real**: `nyash_kernel` imports concrete types from `nyash-rust`
3. **Original goal unclear**: "Same code for plugins and core" not achieved
4. **Trait-based is standard**: Rust pattern for avoiding circular deps
5. **Rename reveals intent**: `nyash_plugin_abi` is more honest name

---

## References

- [Rust circular dependency patterns](https://doc.rust-lang.org/book/ch15-05-interior-mutability.html)
- [Trait objects for runtime polymorphism](https://doc.rust-lang.org/book/ch17-02-trait-objects.html)
- Current code locations:
  - `crates/nyash_kernel/src/lib.rs` (ABI exports)
  - `crates/nyash_kernel/src/plugin/invoke.rs` (plugin shims)
  - `plugins/*/src/lib.rs` (plugin implementations)
