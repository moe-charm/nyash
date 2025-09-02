Title: WASM Backend v2 (Phase 12) – Minimal Handoff

Goal
- Implement a new `src/backend/wasm_v2/` backend that aligns with the unified vtable/slot dispatch model, without touching MIR-level semantics.

Scope
- Allowed:
  - `src/backend/wasm_v2/*` (mod.rs, unified_dispatch.rs, vtable_codegen.rs)
  - `src/backend/wasm/*` read-only for reference
  - `projects/nyash-wasm/*` (demo HTML/JS harness)
  - Documentation under `docs/backend/wasm/*.md`
- Avoid (to keep merge easy):
  - MIR layer (no plugin-name hardcoding)
  - `src/runtime/*` core (HostHandle/host_api/extern are assumed stable)
  - JIT/VM files unless behind feature/cfg guards

Design Constraints
- ExternCall and BoxCall remain separate at the MIR/semantics level.
- At lower layers, converge onto a unified host-call shim when needed (same C symbols as VM/JIT).
- Console/print: use `ExternCall(env.console.log)`; do not move to BoxCall.
- vtable/slots must align with `src/runtime/type_registry.rs` (see Array/Map/String slots).

Targets & Flags
- Target: `wasm32-unknown-unknown`
- Feature: `--features wasm-backend`
- Use `#[cfg(feature = "wasm-backend")]` and/or `#[cfg(target_arch = "wasm32")]` to isolate new code.

Acceptance (Minimal)
- Build succeeds for non-wasm targets (no regressions).
- wasm_v2 compiles behind `--features wasm-backend` (even as stubs).
- A demo harness can call `env.console.log` from WASM and produce a message.
- (Nice to have) Array/Map len/size/has/get stubs go through unified dispatch.

Next Steps (Optional)
- Implement unified dispatch bridge to JS host for `env.console` and basic collections.
- Add a minimal test or demo comparing VM vs WASM return values for a simple program.

