# nyrt.* Externs — String/Array/Map Lowering Map

Purpose
- Define the canonical Extern call surface used by the builder to lower builtin Box methods.
- Provide a single place to verify VM/LLVM parity for core string/collection APIs.

Status
- Phase 15.7: Enabled in VM via `extern_adapter`. Builder lowers a subset today; LLVM harness uses the same registry metadata.

## String (byte semantics)

- nyrt.string.length(recv: String) -> i64
- nyrt.string.indexOf(recv: String, needle: String, from: i64=0) -> i64
- nyrt.string.lastIndexOf(recv: String, needle: String, from: i64=len) -> i64
- nyrt.string.substring(recv: String, start: i64, end: i64) -> String
- nyrt.string.charAt(recv: String, idx: i64) -> String
- nyrt.string.replace(recv: String, from: String, to: String) -> String

Lowering notes
- Methods lowered: `length|size|len`, `substring`, `indexOf` (and alias `find`), `lastIndexOf`, `charAt`, `replace`.
- Receiver is prepended to arguments.
- Semantics implemented in `crates/hako_core_string` and used from both VM and plugins.

## Array

- nyrt.array.size(recv: ArrayBox) -> i64

Lowering notes
- Builder lowers `length|size|len` to `nyrt.array.size`.

## Map

- nyrt.map.size(recv: MapBox) -> i64
- nyrt.map.keys(recv: MapBox) -> ArrayBox
- nyrt.map.values(recv: MapBox) -> ArrayBox

Lowering notes
- `keys/values` return ArrayBox. VM provides a Stage‑1 shim (string list split) and a Stage‑2 path (ArrayBox handle) guarded by env.

## Builder Policy (summary)

- If a builtin method matches the table above, emit `Call{ callee=Extern("iface.method") }` with receiver as the first arg.
- Otherwise, keep as `Method`/`BoxCall` and let VM/Plugin bridge handle semantics.

## VM Adapter

- Implementations live in `src/backend/mir_interpreter/extern_adapter.rs` and delegate to core crates:
  - `hako_core_string`, `hako_core_array`, `hako_core_map`.

## Notes on semantics

- String operations are byte‑based by default. Code‑point aware variants may be added later behind a feature flag.
- Map.keys/values ordering is plugin/builtin defined but must be deterministic in tests.



## Policies

- Externs are the canonical lowering targets for builtin String/Array/Map operations.
- Array.slice is not lowered to Extern today (method path); semantics are defined in `hako_core_array::slice_bounds`.
- Current quick-selfhost profile expects `slice(start, end<0)` to clamp `end` to `len` (full tail).

Array.slice (Stage​‑2 HostHandle)
- When `NYASH_PLUGIN_ARRAY_SLICE_HANDLE=1` is set and the Array plugin is active, `ArrayBox.slice(start,end)` returns a HostHandle (TLV tag=9) pointing to a builtin ArrayBox constructed via host reverse-calls. This unifies the call path so `b.length()`/`toJSON()` use the same builtin semantics and stabilizes smoke outputs.
 - Build requirement: compile the host with `HAKO_EXPORT_HOST=1` (alias: `NYASH_EXPORT_HOST=1`) so reverse-call symbols are exported (adds `-rdynamic` on Linux). Smokes that rely on HostHandle will auto‑skip when symbols are missing.

Debug/Env Gates (aliases available)
- `NYASH_CONSOLE_TRACE` (alias: `HAKO_CONSOLE_TRACE`) → verbose console logging
- `NYASH_DEBUG_TRACE` (alias: `HAKO_DEBUG_TRACE`) → debug.trace prints
- `NYASH_ENABLE_NYKERNEL_STUB` (alias: `HAKO_ENABLE_NYKERNEL_STUB`) → enable nykernel stub externs (dev only)
- Fallback behavior: when the env is unset, the plugin returns a PluginHandle (TLV tag=8) to a new plugin-side Array instance containing the slice.
- Build requirement: compile the host with `HAKO_EXPORT_HOST=1` to export reverse-call symbols (adds `-rdynamic` on Linux).
