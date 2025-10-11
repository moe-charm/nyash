# Plugin Loader V2 Lifecycle

**Status**: Production Ready (Phase 3 Complete)
**Last Updated**: 2025-10-11

## Overview

This document describes the **initialization, runtime, and shutdown lifecycle** of the Plugin Loader V2 system, focusing on global state management and singleton finalization.

## Lifecycle Phases

```
┌─────────────────────────────────────────────────────────────┐
│ Phase 1: Initialization (Lazy)                              │
├─────────────────────────────────────────────────────────────┤
│ 1. HANDLE_CACHE initialization (OnceCell)                   │
│ 2. Plugin library loading (dlopen)                          │
│ 3. TypeBox FFI probing (nyash_typebox_*)                    │
│ 4. Per-plugin INSTANCES initialization (Lazy<Mutex<...>>)   │
│ 5. Singleton pre-birth (optional, env-gated)                │
└─────────────────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────────────────┐
│ Phase 2: Runtime (Normal Operation)                         │
├─────────────────────────────────────────────────────────────┤
│ - Plugin method invocations                                 │
│ - Instance creation/cloning                                 │
│ - HostHandle cache hits/misses                              │
│ - Automatic finalization on Drop                            │
└─────────────────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────────────────┐
│ Phase 3: Shutdown (Explicit or Exit)                        │
├─────────────────────────────────────────────────────────────┤
│ 1. shutdown_singletons() - finalize all cached handles      │
│ 2. Plugin library unloading (automatic via Arc<Library>)    │
│ 3. HANDLE_CACHE cleanup (automatic)                         │
│ 4. INSTANCES cleanup (automatic)                            │
└─────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Initialization

### 1.1 HANDLE_CACHE (Global Singleton)

**Location**: `src/runtime/plugin_loader_v2/enabled/types.rs:235`

```rust
static HANDLE_CACHE: OnceCell<RwLock<HashMap<(u32, u32), Weak<PluginHandleInner>>>>
    = OnceCell::new();

pub fn cache() -> &'static RwLock<HashMap<(u32, u32), Weak<PluginHandleInner>>> {
    HANDLE_CACHE.get_or_init(|| RwLock::new(HashMap::new()))
}
```

**Initialization Trigger**: First call to `cache()` (lazy)

**Purpose**:
- Cache `(type_id, instance_id) → Weak<PluginHandleInner>` mappings
- Prevent duplicate `Arc` allocations for same instance
- Enable handle reuse across multiple Rust references

**Lifetime**:
- Initialized: First plugin Box creation
- Cleanup: Automatic on program exit (no explicit cleanup needed)

**Thread Safety**: `RwLock` - multiple readers, single writer

---

### 1.2 Plugin Library Loading

**Location**: `src/runtime/plugin_loader_v2/enabled/loader/library.rs:23`

```rust
pub(super) fn load_plugin(
    loader: &PluginLoaderV2,
    lib_name: &str,
    lib_def: &LibraryDefinition,
) -> BidResult<()> {
    // 1. Resolve library path
    let lib_path = resolve_library_path(base);

    // 2. dlopen (unsafe)
    let lib = unsafe { Library::new(&lib_path) }?;
    let lib_arc = Arc::new(lib);

    // 3. Call nyash_plugin_init (if present)
    unsafe {
        if let Ok(init_sym) = lib_arc.get::<Symbol<unsafe extern "C" fn() -> i32>>(
            b"nyash_plugin_init\0"
        ) {
            let _ = init_sym();
        }
    }

    // 4. Store LoadedPluginV2
    let loaded = LoadedPluginV2 {
        _lib: lib_arc.clone(),
        box_types: lib_def.boxes.clone(),
    };
    loader.plugins.write()?.insert(lib_name.to_string(), Arc::new(loaded));

    // 5. Probe TypeBox FFI for each box type
    for box_type in &lib_def.boxes {
        probe_typebox(loader, lib_name, box_type, &lib_arc)?;
    }

    Ok(())
}
```

**Load Order**:
- Determined by `hako.toml` `[libraries]` order
- **No dependency resolution** - user must order correctly for Stage-2 plugins
- Future: Automatic topological sort based on dependency graph

**Init Function**: `nyash_plugin_init()` (optional)
- Called once per plugin library load
- Must return `i32` (0 = success)
- Used for plugin-side global state initialization

---

### 1.3 TypeBox FFI Probing

**Location**: `src/runtime/plugin_loader_v2/enabled/loader/specs.rs:76`

```rust
pub(super) fn record_typebox_spec(
    loader: &PluginLoaderV2,
    lib_name: &str,
    box_type: &str,
    tb: &NyashTypeBoxFfi,
) -> BidResult<()> {
    // Validate ABI tag
    if tb.abi_tag != NYASH_TYPEBOX_ABI_TAG {
        return Err(BidError::PluginError);
    }

    // Extract invoke function
    let invoke_fn = tb.invoke_id.ok_or(BidError::PluginError)?;

    // Store in box_specs
    let spec = BoxSpec {
        lib_name: lib_name.to_string(),
        box_type: box_type.to_string(),
        type_id: Some(resolve_type_id(tb)),
        invoke_fn: Some(invoke_fn),
        // ...
    };
    loader.box_specs.write()?.insert((lib_name, box_type), spec);

    Ok(())
}
```

**Probed Symbols**:
1. `nyash_typebox_<BoxType>` - Main TypeBox FFI struct
2. `nyash_typebox_final_<BoxType>` - Final ABI (env-gated, Phase A minimal)

**Storage**: `PluginLoaderV2.box_specs: RwLock<HashMap<(String, String), BoxSpec>>`

---

### 1.4 Per-Plugin INSTANCES Initialization

**Location**: Each plugin's `lib.rs` (via `define_instance_storage!` macro)

```rust
// Example: nyash-array-plugin/src/lib.rs
use hako_abi_impl::define_instance_storage;

struct ArrayInstance {
    data: Vec<Box<dyn NyashBox>>,
}

define_instance_storage!(ArrayInstance);
// Expands to:
//   static INSTANCES: Lazy<Mutex<HashMap<u32, ArrayInstance>>> = Lazy::new(...);
//   static INSTANCE_COUNTER: AtomicU32 = AtomicU32::new(1);
```

**Initialization Trigger**: First instance creation (lazy)

**Lifetime**:
- Initialized: First call to `new()` method
- Cleanup: Automatic on program exit

**Thread Safety**: `Mutex` - single lock for all instances of that Box type

**Scope**: Per-plugin, not shared across plugins

---

### 1.5 Singleton Pre-Birth (Optional)

**Location**: `src/runtime/plugin_loader_v2/enabled/loader/singletons.rs:19`

```rust
pub(super) fn prebirth_singletons(loader: &PluginLoaderV2) -> BidResult<()> {
    if !crate::config::env::plugin_prebirth() {
        return Ok(());  // Disabled by default
    }

    // Create singleton instances for specified boxes
    for (lib_name, box_type) in SINGLETON_BOXES {
        let instance_id = invoke_static_new(loader, lib_name, box_type)?;
        SINGLETON_CACHE.write()?.insert(
            (lib_name.to_string(), box_type.to_string()),
            instance_id
        );
    }

    Ok(())
}
```

**Environment Variable**: `HAKO_PLUGIN_PREBIRTH=1`

**Purpose**:
- Pre-create singleton instances (e.g., `ConsoleBox`, `EnvBox`)
- Avoid allocation overhead on first use
- Enable static initialization in selfhost compiler

**Default**: Disabled (lazy creation is preferred)

---

## Phase 2: Runtime (Normal Operation)

### 2.1 Instance Creation

**Flow**: `new()` → Plugin invoke → Host decode → Cache

```rust
// 1. User code
let arr = new ArrayBox();

// 2. VM/LLVM → Plugin invoke
let result = plugin_invoke(type_id=12, method_id=0, instance_id=0, args);

// 3. Plugin-side (nyash-array-plugin)
pub extern "C" fn nyash_plugin_invoke(...) -> i32 {
    match method_id {
        METHOD_NEW => {
            let id = INSTANCE_COUNTER.fetch_add(1, Ordering::SeqCst);
            INSTANCES.lock().unwrap().insert(id, ArrayInstance::new());
            write_tlv_handle(type_id, id, result, result_len)
        }
    }
}

// 4. Host-side decode (ffi_bridge.rs)
let (type_id, instance_id) = decode_handle(result_tlv);
let inner = get_or_create_handle(type_id, instance_id, invoke_fn, fini_id);
let plugin_box = PluginBoxV2 { box_type: "ArrayBox", inner };

// 5. Cache (types.rs:241)
pub fn get_or_create_handle(...) -> Arc<PluginHandleInner> {
    // Check cache first
    if let Some(weak) = HANDLE_CACHE.read().get(&(type_id, instance_id)) {
        if let Some(arc) = weak.upgrade() {
            return arc;  // ← Cache hit!
        }
    }

    // Create new Arc
    let arc = Arc::new(PluginHandleInner { ... });
    HANDLE_CACHE.write().insert((type_id, instance_id), Arc::downgrade(&arc));
    arc
}
```

**Cache Behavior**:
- **Hit**: Existing `Arc<PluginHandleInner>` upgraded from `Weak` (no new allocation)
- **Miss**: New `Arc` created, `Weak` stored in cache
- **Expired**: `Weak::upgrade()` fails → create new `Arc`

**Performance**: Cache hit avoids:
- `Arc` allocation
- Plugin-side instance lookup
- Reduces memory fragmentation

---

### 2.2 Instance Finalization (Drop)

**Trigger**: `Arc<PluginHandleInner>` refcount reaches 0

```rust
impl Drop for PluginHandleInner {
    fn drop(&mut self) {
        if let Some(fini_id) = self.fini_method_id {
            if !self.finalized.swap(true, Ordering::SeqCst) {
                // Call plugin's finalize method
                let tlv_args: [u8; 4] = [1, 0, 0, 0];
                let _ = invoke_alloc(
                    self.invoke_fn,
                    self.type_id,
                    fini_id,
                    self.instance_id,
                    &tlv_args,
                );
            }
        }
    }
}
```

**Finalization Methods**:
1. **Automatic**: `Drop` trait (when last reference dropped)
2. **Explicit**: `plugin_box.finalize_now()` (idempotent)

**Idempotency**: `AtomicBool` ensures finalization runs exactly once

**Plugin-side**:
```rust
// nyash-array-plugin/src/lib.rs
METHOD_FINALIZE => {
    with_instance_mut!(instance_id, |inst| {
        inst.data.clear();  // Release owned data
        INSTANCES.lock().unwrap().remove(&instance_id);
        HAKO_SUCCESS
    })
}
```

---

## Phase 3: Shutdown

### 3.1 Explicit Shutdown: `shutdown_singletons()`

**Location**: `src/runtime/plugin_loader_v2/enabled/loader/singletons.rs:45`

```rust
pub fn shutdown_singletons() {
    if let Ok(cache) = SINGLETON_CACHE.read() {
        for ((lib_name, box_type), instance_id) in cache.iter() {
            // Finalize each singleton instance
            if let Some(handle) = find_handle_by_instance(*instance_id) {
                handle.finalize_now();
            }
        }
    }
    SINGLETON_CACHE.write().unwrap().clear();
}
```

**When to Call**:
- Before program exit (recommended)
- Between test runs (to reset state)
- Not strictly required (automatic cleanup via `Drop`)

**Purpose**:
- Deterministic finalization order
- Flush buffered I/O (FileBox, ConsoleBox)
- Release external resources (network sockets, file handles)

---

### 3.2 Automatic Cleanup on Exit

**HANDLE_CACHE Cleanup**:
- `OnceCell<RwLock<HashMap>>` - no explicit cleanup needed
- OS reclaims memory on process exit

**INSTANCES Cleanup**:
- `Lazy<Mutex<HashMap>>` - no explicit cleanup needed
- Each plugin's `INSTANCES` is dropped when library unloads

**Plugin Library Unloading**:
- `Arc<libloading::Library>` - automatic when last reference dropped
- Typically at program exit (stored in `PluginLoaderV2.plugins`)

**Order**: Undefined (OS-dependent)

**Risk**: If plugin A depends on plugin B, and B unloads first, calling A's finalize may crash.

**Mitigation**: Use `shutdown_singletons()` for deterministic order.

---

## Global State Summary

| Global State | Type | Location | Init Trigger | Cleanup |
|--------------|------|----------|--------------|---------|
| **HANDLE_CACHE** | `OnceCell<RwLock<HashMap>>` | types.rs:235 | First `cache()` call | Automatic (exit) |
| **INSTANCES** (per-plugin) | `Lazy<Mutex<HashMap>>` | Each plugin | First instance | Automatic (exit) |
| **SINGLETON_CACHE** | `Lazy<RwLock<HashMap>>` | singletons.rs:10 | `prebirth_singletons()` | `shutdown_singletons()` |
| **PluginLoaderV2.plugins** | `RwLock<HashMap<String, Arc<LoadedPluginV2>>>` | types.rs:10 | `load_plugin()` | Automatic (Arc drop) |
| **PluginLoaderV2.box_specs** | `RwLock<HashMap<(String, String), BoxSpec>>` | specs.rs:15 | `record_typebox_spec()` | Automatic (exit) |

---

## Thread Safety Guarantees

### Read-Heavy: `RwLock`
- `HANDLE_CACHE` - many concurrent reads (cache lookups), rare writes (new instances)
- `SINGLETON_CACHE` - read-only after `prebirth_singletons()`
- `PluginLoaderV2.box_specs` - read-only after plugin loading

### Write-Heavy: `Mutex`
- Per-plugin `INSTANCES` - every method call may mutate instance data

### Lock-Free: `AtomicU32`, `AtomicBool`
- `INSTANCE_COUNTER` - monotonic ID generation
- `PluginHandleInner.finalized` - one-time finalization flag

---

## Initialization Order Dependencies

### Safe (No Dependencies)
```
HANDLE_CACHE → Can initialize independently
INSTANCES (per-plugin) → Can initialize independently
```

### Load Order Dependent (Stage-2)
```
ArrayBox plugin load
  ↓
MapBox plugin load (calls nyash_array_new_h)
  ↓
JsonBox plugin load (calls nyash_map_new_h, nyash_array_new_h)
```

**Current Mitigation**: Lazy symbol resolution - `nyash_array_new_h` is not resolved until first call to `MapBox.keys()`.

**Future**: Topological sort of `hako.toml` libraries based on dependency graph.

---

## Environment Variables (Lifecycle Control)

| Variable | Default | Purpose |
|----------|---------|---------|
| `HAKO_PLUGIN_PREBIRTH` | `0` | Enable singleton pre-birth |
| `HAKO_PLUGIN_POLICY` | `auto` | Plugin loading policy (auto/force/off) |
| `NYASH_DEBUG_PLUGIN` | `0` | Enable plugin debug logs |
| `NYASH_HOST_HANDLE_TRACE` | `0` | Trace HostHandle creation/finalization |

**Debug Logging Example**:
```bash
NYASH_DEBUG_PLUGIN=1 ./target/release/hako test.nyash

# Output:
[CACHE MISS] type_id=12 instance_id=1 - creating new Arc
[CACHE INSERT] type_id=12 instance_id=1 cache_size=1
[CACHE HIT] type_id=12 instance_id=1 arc_strong=2
```

---

## Best Practices

### 1. Plugin Development
- ✅ Use `define_instance_storage!` macro (thread-safe by default)
- ✅ Implement `METHOD_FINALIZE` to release resources
- ✅ Make finalization **idempotent** (safe to call multiple times)
- ❌ Don't store raw pointers to other plugins' data
- ❌ Don't call other plugins directly (use HostHandle)

### 2. Host Integration
- ✅ Call `shutdown_singletons()` before exit (deterministic cleanup)
- ✅ Use `NYASH_DEBUG_PLUGIN=1` for cache hit/miss visibility
- ✅ Document Stage-2 dependencies in `hako.toml`
- ❌ Don't assume plugins load in parallel (sequential for now)

### 3. Testing
- ✅ Test with `HAKO_PLUGIN_PREBIRTH=1` to catch singleton issues
- ✅ Verify no leaks with `NYASH_HOST_HANDLE_TRACE=1`
- ✅ Test finalization by dropping all references explicitly
- ❌ Don't rely on finalization order (undefined without `shutdown_singletons()`)

---

## Troubleshooting

### Problem: "symbol not found: nyash_array_new_h"

**Cause**: MapBox plugin loaded before ArrayBox plugin (Stage-2 dependency)

**Fix**: Reorder plugins in `hako.toml`:
```toml
[libraries]
"nyash-array" = { path = "plugins/nyash-array-plugin/target/release/libnyash_array_plugin", boxes = ["ArrayBox"] }
"nyash-map" = { path = "plugins/nyash-map-plugin/target/release/libnyash_map_plugin", boxes = ["MapBox"] }
```

---

### Problem: Cache size grows indefinitely

**Cause**: `Weak` references in `HANDLE_CACHE` are never cleaned up (even when expired)

**Fix**: Periodic cleanup (future feature):
```rust
pub fn cleanup_expired_handles() {
    let mut map = HANDLE_CACHE.write().unwrap();
    map.retain(|_, weak| weak.strong_count() > 0);
}
```

---

### Problem: Finalization not called

**Cause 1**: References still held (Arc refcount > 0)

**Debug**:
```bash
NYASH_DEBUG_PLUGIN=1 ./hako test.nyash
# Look for: arc_strong=N (should be 1 before Drop)
```

**Cause 2**: `fini_method_id` not set

**Fix**: Ensure plugin exports `METHOD_FINALIZE` and host sets `fini_method_id` in BoxSpec.

---

## Future Improvements

### 1. Automatic Dependency Resolution
- Parse `hako_box.toml` `[dependencies]` section
- Topological sort of plugin load order
- Fail-fast on circular dependencies

### 2. Plugin Versioning
- Check `NyashTypeBoxFfi.version` field
- Reject incompatible plugins
- Support multiple ABI versions simultaneously

### 3. Hot Reload
- Unload old plugin library
- Reload new version
- Migrate existing instances (if ABI-compatible)

---

## References

- **Shared ABI Design**: [hakorune-shared-abi-architecture.md](hakorune-shared-abi-architecture.md)
- **Plugin Dependencies**: [plugin-dependency-graph.md](plugin-dependency-graph.md)
- **Host API**: `src/runtime/host_api.rs`
- **Instance Manager Macro**: `crates/hako_abi_impl/src/instance_manager.rs`

---

**Version**: 1.0
**Author**: Claude + ChatGPT (2025-10-11)
