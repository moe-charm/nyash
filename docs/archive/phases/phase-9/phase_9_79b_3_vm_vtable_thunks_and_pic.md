# Phase 9.79b.3: VM VTable Thunks + Poly-PIC (Scaffolding → Production)

Status: Planned (scaffolding in-progress)
Owner: core-runtime
Target: Before Phase 10 (Cranelift JIT mainline)
Last Updated: 2025-08-26

## Goals
- Replace ad-hoc direct calls with a unified vtable+thunk layer for all Box kinds (builtin/user/plugin).
- Upgrade PIC from monomorphic → polymorphic (2–4 entries) per call-site with versioned validity.
- Stabilize method_id (slot) usage end-to-end and make late-bind explicit.
- Provide robust diagnostics (registry dumps, PIC/VT stats) to support JIT handoff in Phase 10.

## Current Baseline (9.79b.1/2 recap)
- Method slots (0..3 universal; user 4+; builtin/plugin seeded) with builder-side method_id emission when resolvable.
- VM fast-paths:
  - Universal thunks (0..3): toString/type/equals/clone.
  - InstanceBox vtable-like caching (function-name cache per slot/arity), plus monomorphic PIC (threshold=8).
  - PluginBoxV2 method_id fast-path (direct invoke_fn with minimal TLV args; fallback to name-based path).
- Cache invalidation: versioned keys by label `BoxRef:Type` (global map; loader/decl-side bump wired).
- Docs updated; builds green.

## Scope (this step)
1) VTable/Thunk layer
- Define `TypeMeta` and per-type thunk table (fixed address, atomic target pointer).
- Uniform entry for builtin/user/plugin: `slot -> thunk -> target`.
- Integrate version into `TypeMeta` for cache validation; connect to global version map.

2) PIC (Polymorphic)
- Extend current mono-PIC to poly (up to 4 entries): `(type_id, version) -> target`.
- Fast-path ladder: universal → vtable(thunk) → PIC(poly) → slow path.

3) Builder/MIR alignment
- Prefer `method_id` emission; introduce explicit late-bind op/path for unresolved calls (keeps current behavior during transition).
- MIRDebugInfo (opt-in) to map id→name for dumps and JIT logs.

4) Diagnostics
- Registry dump: type→id, (type,method)→slot, slot→name (NYASH_REG_DUMP=1).
- PIC/VT stats: per-function summary lines (hit/miss/evict/thresholds) (NYASH_VM_PIC_STATS=1, NYASH_VM_VT_STATS=1).
- Cache version debug: bump events with label/new version (NYASH_CACHE_DEBUG=1).

## Non-Goals
- Full JIT codegen (Phase 10).  
- Full TLV coverage for all plugin arg/ret kinds (incremental; only required for fast-path coverage hotspots).

## Architecture Plan (target)
- UnifiedBoxRegistry
  - type_name → BoxTypeId (stable), `(type_id, method)` → SlotIdx (stable), `TypeMeta{version, vtable_base, slot_count}`.
  - Thunk pool: fixed table of `MethodThunk{target: AtomicPtr<c_void>, sig, flags}`.
- VM dispatch
  - Given `(recv, method_id, args)` → read `TypeMeta` (id, version) → `thunk = vtable[slot]` → `target = thunk.target.load()` → call.
  - PIC adds per-call-site multi-entry check by `(type_id, version)` before slow path.
- Versioning
  - Global label-driven bump (`BoxRef:Type`); epoch for global resets if needed.

## Implementation Steps
1. Add `TypeMeta` + thunk table scaffolding (no-op thunk targets initially).  
2. Migrate InstanceBox path to use thunk table (populate with current function targets).  
3. Migrate PluginBoxV2 path to thunk target (direct invoke_fn target).  
4. Optionally migrate selected builtin methods (hot set) to thunk targets (incremental).  
5. Upgrade PIC to poly entries (size=4) with simple LRU/round-robin.  
6. Add diagnostics (registry dump, PIC/VT stats, cache bump logs).  
7. Add tests (see Test Plan).  
8. Documentation update and developer guide for slots/thunks/PIC.

## Test Plan
- Unit: slot reservation invariants (universal 0..3; override keeps parent slot if inheritance used later).  
- Unit: builder emits method_id for resolvable cases; late-bind path intact for unresolved.  
- VM: universal thunk correctness; InstanceBox second-call via vtable; PIC threshold/upgrade; cache bump invalidation and re-learn.  
- Plugin: method_id fast-path (invoke_fn) arg/ret common kinds; fallback correctness.  
- Perf sanity: ensure fast-paths reduce instruction count vs slow path on micro benchmarks.

## Risks & Mitigations
- Slot drift across reloads: enforce stable slot policy (never reuse freed slots; regen adds at tail).  
- Cache staleness: version-mixed keys + bump triggers already wired; add epoch for global invalidation.  
- Partial type inference: keep late-bind path; add diagnostics to reveal non-resolvable sites.  
- Plugin ABI variability: keep name-based fallback; expand TLV support stepwise; document supported kinds.

## Milestones & Timeline
- M1 (1–2d): TypeMeta/thunk scaffolding + InstanceBox migration + diagnostics framework.  
- M2 (1–2d): Plugin thunk targets + poly-PIC (2–4 entries) + stats.  
- M3 (1d): Builtin hotset to thunk targets + test coverage + docs polish.  
- Handoff: Phase 10 JIT connects to same thunks/PIC; add codegen stubs.

## Exit Criteria (Phase 9.79b.3)
- All Box kinds use vtable+thunk path on hot calls with correct fallbacks.  
- Poly-PIC active with version-aware entries (observed hits in stats).  
- Registry dump + MIRDebugInfo yield consistent id/name/slot mapping.  
- Tests pass; micro benchmarks show expected fast-path usage.

## Open Items / Nice-to-Haves
- TLV encode/decode coverage: Bool/Float/bytes/array/map for plugin fast-path.  
- Fine-grained bump hooks (method-level) if/when method-level updates are supported.  
- Unified late-bind MIR op (explicit), deprecate name-only BoxCall over time.  
- Developer tooling: `nyash --dump-registry` and `--vm-stats verbose` presets.

## Phase 10 (Cranelift JIT) Readiness
- Thunks serve as stable call targets for JIT stubs.  
- PIC structure maps directly to inline cache checks in generated code.  
- Versioning model allows safe invalidation from JIT side.  
- MIR with stable method_id reduces dynamic name lookup in codegen.

---

Notes:
- Current code already includes: method_id slots, universal thunks, InstanceBox VT cache, mono-PIC, Plugin fast-path via method_id, global version invalidation.  
- This step formalizes the structure (TypeMeta+Thunk) and upgrades PIC to poly before Phase 10.

