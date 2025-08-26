# Phase 9.79b.2: VM VTable Thunks + Mono-PIC

Status: Planned
Owner: core-runtime
Target: Before Phase 10 (Cranelift JIT)
Last Updated: 2025-08-26

## Goals
- Implement unified VM BoxCall path via vtable thunks indexed by `MethodId`.
- Add monomorphic inline cache (PIC) at call sites; prepare for polymorphic expansion.
- Keep behavior identical; improve structure and enable JIT lowering.

## Scope
- VM Dispatch
  - Add `TypeMeta` with `vtable_base`, `version`.
  - `execute_boxcall(receiver, method_id, args)`: lookup thunk = `vtable[slot]` and call target.
- PIC (Monomorphic)
  - Per call-site cache: `(type_id, version) → target` fast path with fallback.
  - Counters for hit/miss (debug only) to validate performance wins.
- Plugin safety (stub)
  - Provide thunk replacement and type `version++` API (actual unload handled later with plugin mgr).
- Tests
  - BoxCall correctness across builtin/user/plugin (plugin mocked if needed).
  - PIC hit on repeated calls; miss on version change.
- Docs
  - Update VM README with unified path and PIC diagram.

## Deliverables
- Unified VM BoxCall path (vtable + thunk)
- Monomorphic PIC
- Test coverage for core scenarios

## Non-Goals
- Polymorphic PIC (plan only)
- JIT emission (Phase 10)

## Risks & Mitigations
- Thunk ABI uniformity: define single target signature usable by builtin/VM/plugin shims.
- Cache invalidation: bump `version` on thunk replacement; verify miss logic.

## Timeline
- 2–3 days

## Acceptance Criteria
- All existing tests pass; new VM dispatch tests pass.
- Measurable hit rate on hot call-sites in debug stats.
- No observable behavior change from user code perspective.

## Roll-forward
- Phase 10: Cranelift JIT lowers BoxCall to the same thunks; add poly-PIC and codegen stubs.

