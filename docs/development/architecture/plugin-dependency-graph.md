# Plugin Dependency Graph

**Status**: Phase 3 Complete (11/15 plugins migrated to Shared ABI)
**Last Updated**: 2025-10-11

## Overview

This document maps the dependency relationships between Hakorune plugins, particularly focusing on **Stage-2 dependencies** where one plugin returns instances of another plugin's Box type.

## Dependency Stages

### Stage-1: Independent Plugins (No Dependencies)

These plugins have **zero dependencies** on other plugins and can be loaded in any order:

| Plugin | Box Type | Description |
|--------|----------|-------------|
| `nyash-integer-plugin` | `IntegerBox` | Integer arithmetic operations |
| `nyash-string-plugin` | `StringBox` | String manipulation |
| `nyash-counter-plugin` | `CounterBox` | Simple counter state |
| `nyash-console-plugin` | `ConsoleBox` | Console I/O operations |
| `nyash-nobirth-plugin` | `NoBirthBox` | No-birth lifecycle test box |
| `nyash-path-plugin` | `PathBox` | File path operations |
| `nyash-fixture-plugin` | `FixtureBox` | Test fixture utilities |

**Load Order**: Any order ✅

---

### Stage-2: Dependent Plugins (Cross-Plugin Returns)

These plugins **return instances** of other plugin Box types, creating dependency relationships:

#### 🔗 ArrayBox (Independent, but depended upon)

| Plugin | Box Type | Dependencies | Notes |
|--------|----------|--------------|-------|
| `nyash-array-plugin` | `ArrayBox` | None | **Base dependency** for Map/Json |

**Used By**: MapBox (keys/values), JsonBox (parse arrays)

---

#### 🔗 MapBox → ArrayBox

| Plugin | Box Type | Dependencies | Stage-2 Methods |
|--------|----------|--------------|-----------------|
| `nyash-map-plugin` | `MapBox` | `ArrayBox` | `keys()` → ArrayBox<br>`values()` → ArrayBox |

**Environment Variable**: `HAKO_PLUGIN_MAP_ARRAY_HANDLE=1`

**Return Type**: `HostHandle(Array)` - Managed by host runtime

**Implementation**:
```rust
// nyash-map-plugin/src/lib.rs
pub const METHOD_KEYS: u32 = 3;
pub const METHOD_VALUES: u32 = 4;

// Returns TLV: [TAG_HOST_HANDLE, handle_id (u64)]
match method_id {
    METHOD_KEYS => {
        let array_handle = nyash_array_from_keys(instance);
        write_tlv_host_handle(array_handle, result, result_len)
    }
    METHOD_VALUES => {
        let array_handle = nyash_array_from_values(instance);
        write_tlv_host_handle(array_handle, result, result_len)
    }
}
```

**C API Bridge**:
```rust
// Host-side implementation (hako_kernel)
#[no_mangle]
pub extern "C" fn nyash_array_new_h() -> u64 {
    let array = ArrayBox::new();
    register_host_handle(Box::new(array))
}

#[no_mangle]
pub extern "C" fn nyash_array_push_h(handle: u64, val: *const u8, val_len: usize) {
    let array = get_host_handle_mut::<ArrayBox>(handle);
    let value = decode_tlv(val, val_len);
    array.push(value);
}
```

**Load Order**: ArrayBox → MapBox

---

#### 🔗 JsonBox → ArrayBox + MapBox

| Plugin | Box Type | Dependencies | Stage-2 Methods |
|--------|----------|--------------|-----------------|
| `nyash-json-plugin` | `JsonBox` | `ArrayBox`<br>`MapBox` | `parse()` → ArrayBox \| MapBox<br>`parse_array()` → ArrayBox<br>`parse_object()` → MapBox |

**Environment Variable**: `HAKO_PLUGIN_JSON_HANDLE=1`

**Load Order**: ArrayBox → MapBox → JsonBox

---

#### 🔗 FileBox → StringBox (Potential Stage-2)

| Plugin | Box Type | Dependencies | Stage-2 Methods |
|--------|----------|--------------|-----------------|
| `nyash-filebox-plugin` | `FileBox` | `StringBox` (potential) | `read_to_string()` → StringBox (if implemented) |

**Current Status**: FileBox uses host-side StringBox (not plugin-to-plugin)

**Load Order**: (Independent for now)

---

## Dependency Graph (Visual)

```
Stage-1 (Independent):
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│ IntegerBox  │  │ StringBox   │  │ CounterBox  │
└─────────────┘  └─────────────┘  └─────────────┘

┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│ ConsoleBox  │  │ PathBox     │  │ FixtureBox  │
└─────────────┘  └─────────────┘  └─────────────┘

Stage-2 (Dependent):
┌─────────────┐
│  ArrayBox   │ ← Base dependency
└──────┬──────┘
       │
       ├─────→ ┌─────────────┐
       │       │   MapBox    │
       │       └──────┬──────┘
       │              │
       └──────────────┴─────→ ┌─────────────┐
                               │  JsonBox    │
                               └─────────────┘
```

---

## Migration Status (Phase 3)

### ✅ Migrated to Shared ABI (11/15)

| Plugin | Status | Shared ABI | Instance Manager | TLV Codec |
|--------|--------|------------|------------------|-----------|
| `array` | ✅ Complete | Yes | Macro | hako_abi_impl |
| `map` | ✅ Complete | Yes | Macro | hako_abi_impl |
| `string` | ✅ Complete | Yes | Macro | hako_abi_impl |
| `integer` | ✅ Complete | Yes | - | hako_abi_impl |
| `json` | ✅ Complete | Yes | Macro | hako_abi_impl |
| `filebox` | ✅ Complete | Yes | Macro | Hybrid* |
| `fixture` | ✅ Complete | Yes | Macro | hako_abi_impl |
| `counter` | ✅ Complete | Yes | - | hako_abi_impl |
| `console` | ✅ Complete | Yes | - | hako_abi_impl |
| `nobirth` | ✅ Complete | Yes | - | hako_abi_impl |
| `path` | ✅ Complete | Yes | Macro | hako_abi_impl |

*Hybrid: Uses both hako_abi_impl (common) and local helpers (filebox-specific TLV)

### ⏳ Pending Migration (4/15)

| Plugin | Status | Blocker | Priority |
|--------|--------|---------|----------|
| `net` | Pending | None | P2 |
| `python` | Pending | Complex FFI | P3 |
| `toml` | Pending | None | P2 |
| `regex` | Pending | None | P2 |

---

## Stage-2 Implementation Checklist

When implementing a **Stage-2 dependent plugin** (returns another plugin's Box):

### 1. Environment Variables
- [ ] Define env var: `HAKO_PLUGIN_<NAME>_HANDLE=1`
- [ ] Document in `docs/guides/env-variables.md`
- [ ] Update smoke tests to pass env var

### 2. Host-Side C API
- [ ] Implement `nyash_<box>_new_h() -> u64` in `hako_kernel`
- [ ] Implement all required `nyash_<box>_*_h(handle: u64, ...)` functions
- [ ] Add to `src/runtime/host_api.rs` anchors

### 3. Plugin-Side FFI
- [ ] Declare `extern "C" fn nyash_<box>_new_h() -> u64;`
- [ ] Call host API when building dependent Box
- [ ] Return `TAG_HOST_HANDLE` TLV with handle ID

### 4. Host-Side Decode
- [ ] Handle `TAG_HOST_HANDLE` in `ffi_bridge.rs:294-297`
- [ ] Create `HostHandleBox::new(handle)` wrapper
- [ ] Return as `Box<dyn NyashBox>`

### 5. Testing
- [ ] Smoke test: `plugin_<name>_stage2_vm.sh`
- [ ] Verify host handle reuse (cache hit logs)
- [ ] Test with env var ON/OFF

---

## Known Issues & Gotchas

### 1. Circular Dependency Risk

**Problem**: If Plugin A returns Plugin B's Box, and Plugin B returns Plugin A's Box, we have a circular dependency.

**Mitigation**:
- Design principle: Stage-2 dependencies must be **acyclic** (DAG only)
- ArrayBox/MapBox are intentionally **leaf nodes** (no dependencies)

### 2. Load Order Dependencies

**Problem**: MapBox must load **after** ArrayBox to call `nyash_array_new_h()`.

**Solution**: Current implementation uses **lazy loading** - symbols are resolved at first use, not at load time.

**Risk**: If strict load ordering is enforced in future, document it in `hako.toml`.

### 3. Stage-2 Flag Proliferation

**Problem**: Each Stage-2 feature requires a new env var (`HAKO_PLUGIN_MAP_ARRAY_HANDLE`, etc.)

**Future Improvement**: Consolidate into `HAKO_PLUGIN_STAGE2=array,map,json`.

---

## Future: Stage-3 (Plugin-to-Plugin Direct Calls)

**Stage-3** would allow Plugin A to directly call methods on Plugin B without going through the host runtime.

**Example**:
```rust
// In MapPlugin:
let json_plugin = load_plugin("nyash-json-plugin");
let json_box = json_plugin.invoke_static("parse", args);
```

**Status**: Not implemented. Stage-2 (HostHandle) is sufficient for current needs.

**Decision Point**: Implement Stage-3 only if performance profiling shows significant overhead from HostHandle indirection.

---

## References

- **Shared ABI Design**: [hakorune-shared-abi-architecture.md](hakorune-shared-abi-architecture.md)
- **TLV Codec**: `crates/hako_abi_impl/src/tlv.rs`
- **Host API**: `src/runtime/host_api.rs`
- **FFI Bridge**: `src/runtime/plugin_loader_v2/enabled/ffi_bridge.rs:294-297`

---

**Version**: 1.0
**Author**: Claude + ChatGPT (2025-10-11)
