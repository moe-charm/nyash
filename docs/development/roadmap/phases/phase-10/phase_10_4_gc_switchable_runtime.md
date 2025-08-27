# Phase 10.4 — GC Switchable Runtime (Scaffold)

Status: scaffolded (hooks only)

Goals
- Decouple execution engines from a concrete GC.
- Provide minimal `GcHooks` with `safepoint()` and `barrier(kind)` used by MIR `Safepoint`/`Barrier*`.
- Make runtime supply a pluggable GC via `NyashRuntimeBuilder::with_gc_hooks`.

What’s done (in this repo)
- Added `src/runtime/gc.rs` with `GcHooks` trait, `BarrierKind`, and `NullGc` (no-op).
- `NyashRuntime` now holds `gc: Arc<dyn GcHooks>`; defaults to `NullGc`.
- VM dispatch calls hooks on `Safepoint` and `Barrier(Read|Write|unified)`.

Next
- Thread-local root set API design (`enter_scope/leave_scope`, root pinning) for precise collectors.
- Card marking/write barrier integration for object field writes (`RefSet` sites).
- Preemption policy at safepoints (cooperative scheduling integration).

