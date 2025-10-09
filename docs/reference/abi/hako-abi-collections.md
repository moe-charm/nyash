# Hako ABI — Collections (StringBox, ArrayBox, MapBox)

Status: stable (naming); implementation: plugin or embedded

## Overview

Hako ABI (formerly “Nyash ABI”) defines a small, stable TypeBox interface for the core collection Boxes. The ABI is identical regardless of whether Boxes are provided by dynamic plugins (.so/.dll) or embedded (static) implementations. Runtimes route calls to whichever provider is active; semantics remain the same.

- Boxes: `StringBox`, `ArrayBox`, `MapBox`
- Call surface: TypeBox v2 (resolve → method_id, invoke_id(type_id, method_id, instance_id, TLV-args) → TLV-result)
- Identity: plugin instances are identified by `(type_id, instance_id)`; the host caches handles so reconstructed Boxes reuse the same Arc (pointer-stable, single finalize).

## TLV Value Types (accepted/returned)

- tag=1: Bool (1 byte)
- tag=2: I32 (LE)
- tag=3: I64 (LE)
- tag=5: F64 (LE)
- tag=6/7: String (UTF‑8)
- tag=8: PluginHandle(type_id, instance_id) — host reconstructs PluginBoxV2 (identity shared)
- tag=9: HostHandle(u64) — host maps back to BoxRef (host-managed)

Keys (Map): accept i64 or string (tag=3 or 6/7). Keys are internally normalized to string to guarantee ordering and JSON determinism.

## Semantics (unified)

Common
- `size() -> IntegerBox`
- `isEmpty() -> BoolBox` (convenience; equivalent to `size()==0`)

StringBox
- `size() -> IntegerBox`
- `indexOf(substr) -> IntegerBox` (−1 if not found)
- `lastIndexOf(substr) -> IntegerBox` (−1 if not found)
- `substring(start, end) -> StringBox`
- `charAt(index) -> StringBox` (empty string if OOB)
- `toUpper()/toLower()/trim()/toInteger()` …

ArrayBox
- `size()/length() -> IntegerBox` (length kept as alias)
- `get(index) -> Box | NullBox` (OOB → NullBox)
- `set(index, value) -> NullBox` (index==len appends)
- `push(value) -> NullBox`, `pop() -> Box | NullBox`, `remove(index) -> Box | NullBox`
- `indexOf(value) -> IntegerBox(−1)`
- `contains(value) -> BoolBox`
- `slice(start, end) -> ArrayBox` (bounds clamped)
- `clear() -> NullBox`, `toJSON() -> StringBox`

MapBox
- `size() -> IntegerBox`, `isEmpty() -> BoolBox`
- `get(key) -> Box | NullBox` (missing → NullBox)
- `set(key, value) -> NullBox`
- `delete(key) -> NullBox`
- `has(key) -> BoolBox`
- `keys() -> ArrayBox<StringBox>` (sorted for determinism)
- `values() -> ArrayBox<Box>` (may include tag=8/9 handles; identity preserved)
- `clear() -> NullBox`
- Optional: `getOr(key, default) -> Box`

## Plugin vs Embedded

- Dynamic plugin: loader resolves `(type_id, method_id)` from `nyash.toml`/`hako.toml`, and calls through `invoke_id`.
- Embedded: host registers equivalent provider; the registry/runner routes by policy.
- Factory policy: `StrictPluginFirst` (default in dev) or `BuiltinFirst` (plugins disabled). Behavior is identical.

## Identity and Lifecycle

- For tag=8 (PluginHandle), the host maintains a global cache `(type_id, instance_id) → Weak<PluginHandleInner>`; reconstructing a handle upgrades or creates a single Arc. This guarantees pointer-stable identity and single `fini` call.
- For tag=9 (HostHandle), the host maps the number back to a managed BoxRef.

## JSON and Ordering

- `keys()` returns sorted keys (string order). `toJSON()` uses the same ordering for deterministic output.

## Naming

- “Hako ABI” is the canonical name. Code and older docs may still refer to “Nyash ABI” or “TypeBox v2”; these are aliases. ABI contracts remain identical.

