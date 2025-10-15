# Hako ABI Architecture: Solution Options Comparison

## Current Problem: Circular Dependency

```
┌──────────────────────┐
│    nyash-rust        │
│  (Core Runtime)      │
│                      │
│  • NyashBox trait    │
│  • host_handles      │
│  • plugin_loader_v2  │
│  • Box types         │
└──────────────────────┘
           ▲
           │
           │ CIRCULAR DEP! ❌
           │
           ▼
┌──────────────────────┐
│   nyash_kernel       │
│  (Shared ABI Layer)  │
│                      │
│  Uses:               │
│  • NyashBox          │
│  • host_handles      │
│  • plugin_loader_v2  │
└──────────────────────┘
```

---

## Option 1: Trait-Based Abstraction ⭐ (RECOMMENDED)

### Architecture:

```
┌─────────────────────────────────┐
│    nyash_runtime_traits         │
│    (Pure Traits - No Impl)      │
│                                  │
│  pub trait HandleRegistry {     │
│    fn get(&self, h: u64) -> ... │
│    fn to_handle(...) -> u64     │
│  }                              │
│                                  │
│  pub trait BoxTrait {           │
│    fn as_any(&self) -> &dyn Any │
│    fn to_string_box() -> ...    │
│  }                              │
│                                  │
│  pub trait PluginHost {         │
│    fn create_box(...) -> ...    │
│  }                              │
└─────────────────────────────────┘
           ▲                ▲
           │                │
           │                │
    ┌──────┴──────┐  ┌──────┴──────────┐
    │             │  │                 │
┌───▼─────────────┐  │  ┌──────────────▼───┐
│ nyash_plugin_abi│  │  │   nyash-rust     │
│ (Trait Users)   │  │  │ (Trait Impls)    │
│                 │  │  │                  │
│ Uses:           │  │  │ Implements:      │
│ • dyn Registry  │  │  │ • HandleRegistry │
│ • dyn BoxTrait  │  │  │ • BoxTrait       │
└─────────────────┘  │  │ • PluginHost     │
                     │  └──────────────────┘
                     │
                     │  ┌──────────────────┐
                     └─▶│    Plugins       │
                        │                  │
                        │ Use via traits   │
                        └──────────────────┘
```

### Pros:
- ✅ **Zero-cost abstraction**: Trait dispatch is compile-time optimized
- ✅ **Type safety**: Full Rust type checking
- ✅ **No circular deps**: Traits don't depend on implementations
- ✅ **Standard Rust pattern**: Widely used and understood
- ✅ **Testable**: Easy to mock traits

### Cons:
- ⚠️ **Refactoring effort**: ~2-3 days to extract traits
- ⚠️ **More crates**: Need `nyash_runtime_traits` crate
- ⚠️ **Trait bounds**: Complex generic signatures possible

### Implementation Estimate:
**2-3 days** for full migration

---

## Option 2: C-Compatible ABI Layer

### Architecture:

```
┌─────────────────────────────────┐
│       nyash_abi_ffi             │
│    (C-Compatible Types)         │
│                                  │
│  #[repr(C)]                     │
│  pub struct HandleId(u64);      │
│                                  │
│  extern "C" fn                  │
│  nyash_handle_get(              │
│    h: HandleId                  │
│  ) -> *const u8;                │
│                                  │
│  extern "C" fn                  │
│  nyash_box_new(                 │
│    type_name: *const c_char     │
│  ) -> HandleId;                 │
└─────────────────────────────────┘
           ▲                ▲
           │                │
           │                │
    ┌──────┴──────┐  ┌──────┴──────────┐
    │             │  │                 │
┌───▼─────────────┐  │  ┌──────────────▼───┐
│ nyash_plugin_abi│  │  │   nyash-rust     │
│ (C ABI Wrapper) │  │  │ (Rust Runtime)   │
│                 │  │  │                  │
│ Wraps C ABI     │  │  │ Implements       │
│ with safety     │  │  │ C ABI funcs      │
└─────────────────┘  │  └──────────────────┘
                     │
                     │  ┌──────────────────┐
                     └─▶│    Plugins       │
                        │  (C or Rust)     │
                        └──────────────────┘
```

### Pros:
- ✅ **Language agnostic**: Works with C, C++, etc.
- ✅ **Stable ABI**: No Rust version coupling
- ✅ **Maximum decoupling**: No shared types
- ✅ **Cross-FFI safe**: Works across DLL boundaries

### Cons:
- ⚠️ **Manual safety**: Unsafe blocks everywhere
- ⚠️ **Memory management**: Manual lifetime tracking
- ⚠️ **Verbose**: More boilerplate code
- ⚠️ **Lost type safety**: C types don't enforce Rust semantics

### Implementation Estimate:
**3-4 days** for safe wrappers

---

## Option 3: Global Registry Pattern

### Architecture:

```
┌─────────────────────────────────┐
│      nyash_plugin_abi           │
│                                  │
│  static RUNTIME: OnceLock<      │
│    Box<dyn RuntimeBridge>       │
│  > = OnceLock::new();           │
│                                  │
│  pub fn init(rt: Box<...>) {    │
│    RUNTIME.set(rt)              │
│  }                              │
│                                  │
│  pub fn get_handle(h: u64) {    │
│    RUNTIME.get()?.get_handle(h) │
│  }                              │
└─────────────────────────────────┘
                    ▲
                    │ provides at runtime
                    │
                    │
┌───────────────────▼─────────────┐
│         nyash-rust              │
│                                  │
│  struct MyRuntime { ... }       │
│                                  │
│  impl RuntimeBridge for         │
│    MyRuntime { ... }            │
│                                  │
│  fn main() {                    │
│    nyash_plugin_abi::init(      │
│      Box::new(MyRuntime)        │
│    )                            │
│  }                              │
└─────────────────────────────────┘
```

### Pros:
- ✅ **Minimal changes**: Add init + global state
- ✅ **Runtime flexible**: Can swap implementations
- ✅ **No circular deps**: Runtime injected at startup

### Cons:
- ⚠️ **Global state**: Testing harder (need reset)
- ⚠️ **Init required**: Panics if not initialized
- ⚠️ **Single runtime**: Hard to have multiple runtimes
- ⚠️ **Lifetime issues**: Global ownership tricky

### Implementation Estimate:
**1-2 days** for basic version

---

## Option 4: Separate ABI Crate (Quick Fix)

### Architecture:

```
┌─────────────────────────────────┐
│       nyash-rust                │
│     (Core Runtime)              │
│                                  │
│  • NyashBox                     │
│  • host_handles                 │
│  • plugin_loader_v2             │
└─────────────────────────────────┘
                    │
                    │ depends on
                    ▼
┌─────────────────────────────────┐
│      nyash_abi_core             │
│  (ABI Implementations)          │
│                                  │
│  • All ABI functions            │
│  • Uses nyash-rust types        │
└─────────────────────────────────┘
           ▲                ▲
           │                │
           │ re-exports     │ re-exports
           │                │
    ┌──────┴──────┐  ┌──────┴──────────┐
    │             │  │                 │
┌───▼─────────────┐  │  ┌──────────────▼───┐
│ nyash_plugin_abi│  │  │   nyash-rust     │
│ (re-export)     │  │  │  (re-export)     │
└─────────────────┘  │  └──────────────────┘
                     │
                     │  ┌──────────────────┐
                     └─▶│    Plugins       │
                        └──────────────────┘
```

### Pros:
- ✅ **Quick to implement**: Just reorganize crates
- ✅ **Preserves API**: No breaking changes
- ✅ **Technically works**: Compiles successfully

### Cons:
- ⚠️ **Doesn't solve root cause**: Just hides circular dep
- ⚠️ **Confusing**: Which crate is "real" source?
- ⚠️ **Not truly shared**: Still coupled to runtime

### Implementation Estimate:
**4-6 hours** for crate shuffle

---

## Option 5: Accept Plugin-Only Reality ⭐ (PRAGMATIC)

### Architecture:

```
┌─────────────────────────────────┐
│         nyash-rust              │
│      (Core Runtime)             │
│                                  │
│  Used by:                       │
│  • AOT binaries (direct link)   │
│  • Core runtime                 │
│  • VM interpreter               │
└─────────────────────────────────┘
                    │
                    │ provides helpers
                    ▼
┌─────────────────────────────────┐
│    nyash_plugin_abi             │
│   (Plugin Helper Layer)         │
│                                  │
│  • Depends on nyash-rust ✓      │
│  • Provides plugin conveniences │
│  • NOT used by core (honest!)   │
└─────────────────────────────────┐
                    ▲              │
                    │              │
                    │ optional     │ core uses
                    │              │ nyash-rust
                    │              │ directly
┌───────────────────┴────┐  ┌──────▼──────────┐
│       Plugins          │  │   AOT/Core      │
│                        │  │                 │
│  Can use:              │  │  Links:         │
│  • nyash_plugin_abi    │  │  • nyash-rust   │
│  • Own implementations │  │    (direct)     │
└────────────────────────┘  └─────────────────┘
```

### Pros:
- ✅ **Honest**: Reflects actual usage
- ✅ **No circular dep**: Plugin layer depends on core (one-way)
- ✅ **Minimal disruption**: Rename + documentation
- ✅ **Pragmatic**: Solves immediate problem
- ✅ **Future-proof**: Can add traits later

### Cons:
- ⚠️ **Abandons original goal**: Not truly "shared" code
- ⚠️ **Plugins don't use it yet**: Need to update plugins

### Implementation Estimate:
**2-3 hours** for rename + docs

---

## Recommended Hybrid Approach

### Phase 1: Accept Reality (Immediate - 2-3 hours)

1. **Rename**: `nyash_kernel` → `nyash_plugin_abi`
2. **Document**: "Plugin helper layer, not shared runtime"
3. **Update**: Cargo.toml, imports, docs
4. **Test**: Ensure builds work

### Phase 2: Add Traits (Future - if sharing needed)

1. **Create**: `nyash_runtime_traits` crate
2. **Define**: Core traits for abstraction
3. **Implement**: Traits in `nyash-rust`
4. **Update**: `nyash_plugin_abi` to use traits
5. **Migrate**: Plugins to use trait-based ABI

---

## Decision Matrix

| Option | Complexity | Time | Type Safety | ABI Stability | Shared Code |
|--------|-----------|------|-------------|---------------|-------------|
| **1. Traits** | Medium | 2-3 days | ✅ High | ✅ Good | ✅ Yes |
| **2. C ABI** | High | 3-4 days | ⚠️ Low | ✅ Excellent | ✅ Yes |
| **3. Global** | Low | 1-2 days | ✅ High | ⚠️ Runtime | ⚠️ Partial |
| **4. Re-export** | Very Low | 4-6 hours | ✅ High | ⚠️ Coupled | ❌ No |
| **5. Accept** | Minimal | 2-3 hours | ✅ High | ✅ Good | ❌ No |

---

## Conclusion

**Immediate**: Go with **Option 5** (Accept Reality)
- Renames `nyash_kernel` → `nyash_plugin_abi`
- Documents actual purpose
- Solves circular dependency NOW

**Future** (if true sharing needed): Add **Option 1** (Traits)
- Creates `nyash_runtime_traits`
- Enables true code sharing
- Zero-cost abstraction

**Key Insight**: The original goal ("same code for plugins and core") was noble but not actually achieved. Current plugins don't use `nyash_kernel` at all. Being honest about this and renaming the crate solves the immediate problem while leaving the door open for future trait-based sharing if needed.
