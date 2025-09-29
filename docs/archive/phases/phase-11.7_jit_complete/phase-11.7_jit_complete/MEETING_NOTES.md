# Design Notes – Phase 11.7 JIT Complete (Meeting Summary)

Date: 2025-09-01

Key Decisions

- Single Semantics Source: Introduce a MIR semantics layer (trait) as the single source of truth. All backends (VM/Cranelift/LLVM/WASM) implement this interface.
- No Runtime Fallback: Remove VM→JIT fallback complexity. VM becomes the reference executor; codegen backends handle execution/AOT. JIT is compile-only/AOT‑assist when needed.
- Handle‑First ABI: Unify handle/i64/ptr conversions, tag classification, concat/extern/boxcall via shared helpers; call into NyRT shims from backends.
- GC Hooks: Insert sync barriers and async safepoints as MIR‑level hooks that each backend lowers appropriately.
- Backends Roadmap: LLVM AOT は一旦アーカイブ。Cranelift を主線（JIT/軽量AOT）とし、WASM は同一セマンティクスで後段。Windows DX を軽く保つ。

Architecture Sketch

- MIR → Semantics<E>
  - VmSem: executes values (reference)
  - ClifSem: builds Cranelift IR (JIT/AOT)
  - LlvmSem: builds LLVM IR (AOT)
  - WasmSem: builds Wasm (future)
- Shared ABI utilities: handle↔i64/ptr, to_bool, compare, tags, invoke variants, NyRT shims.

Implementation Plan (Progressive)

- Phase 1: Skeleton + minimal lowering (Const/Return/Add) → echo-lite returns 0 via Cranelift JIT skeleton.
- Phase 2: Control (Jump/Branch/Phi), Load/Store, Compare, String concat via NyRT, Box/Extern by-id (fixed/vector).
- Phase 3: GC barriers/safepoints; parity with VM（CountingGc での観測を含む）。
- Phase 4: Stability, logs, strict/legacy guards; optional AOT via cranelift-object + link scripts.

Notes from Review

- Using Semantics trait enables zero‑cost abstractions with static dispatch.
- Add optional debug hooks (location/value) and optimization hints (likely_branch, pure_function_hint) later.
- Testing: MockSemantics for unit tests; parity tests VM vs CLIF.

Action Items

- Land Semantics trait + minimal MirInterpreter (skeleton added).
- Implement ClifSem minimal lowering; wire Runner `--backend cranelift`.
- Centralize ABI helpers; migrate existing scattered logic to shared module.
- Emit `nyash.rt.checkpoint` and `nyash.gc.barrier_write` from LowerCore at appropriate sites; wire Await (PoC: blocking get).
- Add build_cl scripts for AOT when ready.

Links

- PLAN.md – milestones and coverage
- CURRENT_TASK.md – immediate focus
