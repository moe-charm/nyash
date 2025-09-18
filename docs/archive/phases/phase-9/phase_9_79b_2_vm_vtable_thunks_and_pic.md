# Phase 9.79b.2: VM VTable Thunks + Mono-PIC

Status: Planned
Owner: core-runtime
Target: Before Phase 10 (Cranelift JIT)
Last Updated: 2025-08-26
Progress: Fast-path thunks (universal slots) + PIC skeleton committed

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

## Progress Notes (2025-08-26)
- Implemented fast-path thunks for universal slots (0..3: toString/type/equals/clone) in `execute_boxcall` using `method_id`.
- `MirInstruction::BoxCall` now carries optional `method_id` emitted by the builder when resolvable.
- Added monomorphic PIC skeleton: per-(receiver type, method_id/name) hit counters in the VM.
- Minimal tests: verify fast-path behavior for `type()` and `equals()`.
 - PIC direct-call (InstanceBox): after threshold (8 hits), call `{Class}.{method}/{arity}` directly via cache.

Next:
- Threshold-based direct dispatch using per-site cache entries.
- Extend beyond universal slots to general `method_id`-resolved methods (builtin/plugin + user) via vtable thunks.

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
