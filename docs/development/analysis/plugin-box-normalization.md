# Plugin Box Normalization — Findings and Plan (Phase 20.5)

Status: analysis complete (no code changes yet)

## Scope
- Plugin Boxes and call paths that can bypass consistent Box shapes or return raw/partial values.
- Components in scope:
  - HostHandleRouter: `src/runtime/host_handle_router/mod.rs`
  - Method router (plugin path): `src/runtime/method_router_box/plugin.rs`
  - Plugin loader v2 (FFI bridge/externs):
    - `src/runtime/plugin_loader_v2/enabled/ffi_bridge.rs`
    - `src/runtime/plugin_loader_v2/enabled/extern_functions/mod.rs`
  - Box registry / method router (builtin arms, gating): `src/runtime/method_router_box/mod.rs`
  - Static runtime anchors (AOT line): `crates/hako_kernel/src/*`

## Symptoms observed
- Return-shape inconsistency across plugin vs builtin:
  - Some plugin methods return raw host values/handles instead of Box values.
  - `toJSON` is missing for some Map/Array-like values; stringify paths hit `null` (no keys/size).
  - Map semantics (`missing` vs `null`) differ between HostHandle early path and plugin fallback.
- Error code drift/noise:
  - Mixed use of error codes (-1/-11/-13/-14) and plain error strings in different paths.
  - Plugin preflight noise leaks into test parsing unless filtered.

## Root causes (by area)
- HostHandleRouter
  - Early slots for Array/Map/String are good, but plugin fallback paths sometimes return non‑normalized shapes.
  - ENV gating (e.g., `NYASH_MAP_FORCE_HOST`, `NYASH_ARRAY_SIZE_FORCE_HOST`) creates differing shapes in quick vs plugin profiles.
- Plugin loader v2
  - `ffi_bridge.rs` and `extern_functions/mod.rs` contain direct references to BufferBox/FloatBox and friends that are not always normalized for plugin‑first runs.
  - Some externs/pathways yield raw literals/handles.
- Method router (builtin vs plugin)
  - Legacy builtin arms are gated, but plugin path still needs to guarantee identical shapes for Map/Array/String core operations.

## Normalization contract (target)
- Return types must be concrete Box values (e.g., IntegerBox/StringBox/MapBox/ArrayBox/VoidBox) or explicit HostHandle wrappers.
- `toJSON` must be present and stable for Map/Array‑like values used by stringify.
- Missing vs null semantics are consistent:
  - Map.get(missing) → `null` (documented); Map.has → `0/1`.
- Error mapping unified:
  - `-11` unknown handle or receiver type mismatch
  - `-13` TLV decode/arity/tag errors
  - `-14` type mismatch (argument/receiver)

## Plan (small steps, flagged where needed)
1) Define a thin normalization helper in plugin path (no behavior change yet):
   - Input: any host return; Output: Box or explicit HostHandle wrapper.
   - Document the mapping table and error conversion.
2) Fill shape gaps in `ffi_bridge.rs` and `extern_functions/mod.rs`:
   - Ensure BufferBox/FloatBox usage is cfg‑guarded; plugin‑only fallback returns normalized shapes.
3) Align HostHandleRouter and plugin fallback for Map/Array core ops:
   - size/has/get/set return identical shapes (numbers/null for get).
4) Tests (plugins profile):
   - Minimal parity smokes for Map.size/has/get/set & Array.size vs builtin path.
   - Boundary: TLV wrong tag → `-13`, unknown handle → `-11`.

## Notes on profiles and env
- Keep quick profile minimal (only essential force flags). Plugins profile can exercise broader HostHandle paths.
- Maintain SKIP when preconditions are unmet; SMOKES_STATUS/SMOKES_ERR lines are now normalized for parsing.

## References (files)
- HostHandleRouter: `src/runtime/host_handle_router/mod.rs`
- Plugin router path: `src/runtime/method_router_box/plugin.rs`
- Plugin loader v2: `src/runtime/plugin_loader_v2/enabled/ffi_bridge.rs`, `src/runtime/plugin_loader_v2/enabled/extern_functions/mod.rs`
- Builtin router arms (gated): `src/runtime/method_router_box/mod.rs`
- Static runtime anchors: `crates/hako_kernel/src/*`

## Next steps (implementation, separate PR)
- Introduce normalization helper (plugin path) with no behavior change for quick; gate with a feature/env.
- Add 2–3 parity smokes (plugins profile) for Map/Array core ops.
- Iterate on any failing cases and extend the mapping table accordingly.

