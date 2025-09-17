# Phase 11.7 – JIT Complete Plan (Cranelift)

Goal

- Ship a complete JIT backend (Cranelift) for MIR Core‑15 with Semantics layer as the single source of truth, GC sync/async support, and full Box/Extern integration. Keep DX simple and cross‑platform.

Milestones

1) Bootstrap (Infra + Skeleton)
- Add backend module: `src/backend/cranelift/{mod.rs,context.rs,lower.rs,jit.rs,object.rs}`
- Context: host ISA, FunctionBuilder, Module (JIT/Object) setup helpers
- Runner: `--backend cranelift` execution path; feature flag reuse (`cranelift-jit`)
- Minimal ops: Const i64/f64/bool/null/string → CLIF values; Return; straight‑line add/sub
- Smoke: `apps/tests/ny-echo-lite` returns 0 via JIT

2) Core MIR‑15 Lowering (Parity with VM)
- Control: Jump/Branch/Phi, Load/Store (alloca on entry; i1↔i64 widen/narrow)
- Unary/Binary/Compare: int/float/ptr eq/ne; logical and/or via to_bool
- TypeOp (when needed) + pointer/int casts (handle→i64/i8* via ABI helpers)
- Strings: concat via NyRT shims (`nyash.string.concat_*`)
- BoxCall (by‑id): fixed/tagged args (<=4) + vector path; handle‑first returns (i64/ptr)
- ExternCall: `env.console.*`, `env.debug.trace`, `console.readLine` via NyRT shims
- Smokes: array/map/vinvoke(size/ret), echo; compare outputs with VM

3) GC Cooperation
- Sync barriers: insert read/write barriers at Load/Store & NewBox as per VM semantics
- Async safepoints: at call sites/loop backedges; NyRT entry glue to yield if required
- Tests: targeted barrier smoke (array/map mutations) + perf sanity (no excessive barriers)

4) Parity + Stability
- UnsupportedLegacyInstruction: maintain strict mode; allow env override for debug
- Error reporting: source op pretty‑print, MIR value ids in error messages
- Logging: `NYASH_CLI_VERBOSE=1` shows JIT compile stages + object sizes (optional)
- Doc/Doctor script: `tools/doctor.ps1`/`.sh` for quick env checks (optional nice‑to‑have)

5) AOT (Optional within 11.7 if time)
- `cranelift-object` emit: `.o` for main; link with NyRT → exe (`tools/build_cl.*`)
- Windows: clang/lld link; Linux: cc link; parity with LLVM’s scripts

Instruction Coverage (MIR Core‑15)

- Const, UnaryOp, BinOp, Compare, TypeOp
- Load, Store, Phi, Jump, Branch, Return
- Call, NewBox, BoxCall, ExternCall

ABI & Shims

- Handle‑first: receiver/values normalized to i64; ptr/int casts via helpers
- NyRT shims used: `nyash.console.*`, `nyash.debug.trace`, `nyash.console.readline`,
  `nyash_string_new`, `nyash.string.concat_*`, `nyash_array_*_h`, `nyash.instance.*_h`,
  plugin invoke (by‑id tagged, vector variants)

Semantics Integration (new in 11.7)

- Add `Semantics` trait as unified MIR semantics API.
- Provide `SemanticsVM` (exec) and `SemanticsClif` (lower) so the same MIR walks yield identical behavior across VM and JIT.
- Use `semantics::MirInterpreter` for parity tests; prefer zero‑cost abstractions with static dispatch.

Status Notes (2025‑09‑01)

- LLVM AOT: closed for now due to Windows dependency weight; Cranelift is the mainline.
- VM: safepoint/barrier/scheduler wired and observable (CountingGc).
- JIT: `nyash.rt.checkpoint`/`nyash.gc.barrier_write` symbols are available via NyRT; LowerCore needs to emit safepoints and barriers; Await lowering pending.

Deliverables

- Code: Cranelift backend files + runner integration
- Tools: build/run scripts for JIT/AOT; updated smokes to exercise CL route
- Docs: this plan + CURRENT_TASK.md; brief README how to run JIT

Risks & Mitigations

- Pointer/int mismatch → normalize via i64/ptr helpers in one place; add asserts
- Barrier placement overhead → start with conservative placement; measure; trim if safe
- Windows toolchain variance → Cranelift avoids external LLVM; keep MSVC only

Timeline (indicative)

- Week 1: Milestone 1 + 2 (most ops) → basic smokes green
- Week 2: GC barriers + safepoints, full parity sweep; docs/tools polish
- Optional: AOT via cl‑object emit & link scripts
