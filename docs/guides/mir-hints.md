# MIR Hints — Config/Detectors/Applier (Phase 2)

Purpose
- Optimize hot call patterns without changing semantics.
- Start from self-recursive direct calls (Phase 1), then layer tail/pure/escape (Phase 2).

Boxes (Phase 2 skeleton)
- OptimizationConfigBox: `src/mir/optimizer_passes/hints_config.rs`
  - Reads ENV once, exposes `is_enabled(OptimizationType)`
  - Flags: `NYASH_MIR_SELFREC_DIRECT`, `NYASH_MIR_HINTS=tail|pure|escape|all`, `NYASH_MIR_OPTIMIZE_TRACE`
- PatternDetectorBox: `src/mir/optimizer_passes/detectors/`
  - `SelfRecursionDetector`, `TailCallDetector` (extensible trait)
- HintsBuilderBox: `src/mir/optimizer_passes/hints.rs`
  - Builds `HintsMap((fn,idx)->CallHints)` from MIR functions
  - `to_json()` for observability
- LLVMOptimizationApplierBox (planned): `src/llvm_py/optimizer/applier.py` (future)
  - Applies hints: tail/inline/readonly; reports via Reporter
- OptimizationReporterBox: `src/mir/optimizer_passes/reporter.rs`
  - Records applied optimizations; outputs JSON

Phase 1 (implemented)
- Self-recursive direct call (env-gated):
  - Builder: `src/mir/builder/builder_calls/build.rs` (force ModuleFunction for self)
  - LLVM: `src/llvm_py/instructions/mir_call.py` (Global self → direct call)
  - Enable: `NYASH_MIR_SELFREC_DIRECT=1`
  - Trace: `NYASH_MIR_OPTIMIZE_TRACE=1` → one-line JSON

Phase 2 (next)
- Tail: set tail flag for calls identified by TailCallDetector
- Pure/Escape: attach `alwaysinline`/`readonly` or allow DCE under `-O3`
- Reporter: emit minimal JSON for applied events

Notes
- Defaults are OFF; features are opt-in and reversible.
- Hints complement but do not replace structural fixes (Phase 1 stops generating unnecessary boxes at the source).

