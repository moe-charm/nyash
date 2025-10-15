# Hako ABI — Collections (String/Array/Map)

Status: Active (formerly “Nyash ABI”)

Scope
- Single ABI face for core collections implemented as plugins or embedded providers.
- Box types (core): `MapBox` (type_id=11), `ArrayBox` (type_id=12), `StringBox` (type_id=10), `SetBox` (type_id=15).

Principles
- Everything is Box: same lifecycle (`birth`/`fini`) across host/plugin/user.
- Handle identity: TLV tag=8 `(type_id,u32 instance_id)` preserves identity via host handle cache.
- Returns unify to Null where appropriate (e.g., `Map.get()` on missing → `null`).

Minimal TLV
- Args header: `u16 version=1`, `u16 argc`.
- Entries: tag(1) rsv(1) len(u16) payload(len).
- Tags: 1=Bool, 2=I32, 3=I64, 5=F64, 6=String(UTF‑8), 7=Bytes, 8=PluginHandle, 9=HostHandle.

Required methods (subset)
- All collections: `size() -> IntegerBox`, `isEmpty() -> BoolBox`.
- String: `length/size`, `substring(start,end)`, `indexOf(s)`, `lastIndexOf(s)`, `charAt(i)`.
- Array: `get(i) -> Box|null`, `set(i,v) -> null`, `push(v) -> null`.
- Map: `get(k) -> Box|null`, `set(k,v) -> null`, `delete(k) -> null`, `keys() -> ArrayBox`, `values() -> ArrayBox`.
 - Set: `add(v) -> null`, `remove(v) -> null`, `has(v) -> BoolBox`, `toArray() -> ArrayBox`.

Interop notes
- Host bridges and VM dispatch must not special‑case collections; prefer PluginHost facade.
- When plugins are OFF, embedded providers keep behavior equivalent.

Notes
- SetBox is Map‑backed (Map<Key, Unit>) and shares Map’s Eq/Hash/決定モードの意味論。Extern 経路は `nyrt.set.*` に統一される。
