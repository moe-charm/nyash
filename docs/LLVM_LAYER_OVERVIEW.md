# LLVM Layer Overview (Phase 15)

Scope
- Practical guide to LLVM lowering architecture and invariants used in Phase 15.
- Complements LOWERING_LLVM.md (rules), RESOLVER_API.md (value resolution), and LLVM_HARNESS.md (harness).

Module Layout
- `src/backend/llvm/compiler/codegen/`
  - `instructions/` split by concern: `flow.rs`, `blocks.rs`, `arith.rs`, `arith_ops.rs`, `mem.rs`,
    `strings.rs`, `arrays.rs`, `maps.rs`, `boxcall/`, `externcall/`, `call.rs`, `loopform.rs`, `resolver.rs`.
  - `builder_cursor.rs`: central insertion/terminator guard.

Core Invariants
- Resolver-only reads: lowerers fetch MIR values through `Resolver` (no direct `vmap` access for cross-BB values).
- Localize at block start: PHIs created at the beginning of the current BB (before non-PHI) to guarantee dominance.
- Cast placement: perform ptr↔int and width casts outside PHIs, at BB start or just-before-pred-terminator via `with_block`.
- Sealed SSA: successor PHIs wired by predecessor snapshots and `seal_block`; branch/jump do not push incoming directly.
- Cursor discipline: only insert via `BuilderCursor`; post-terminator insertions are forbidden.

LoopForm (gated)
- Shape: `preheader → header → body → dispatch(phi) → {latch|exit} → header` with PHIs centralized in `dispatch`.
- State: model loop-carried values via a `LoopState` aggregate (tag + payloads).
- Goals: move all PHIs to dispatch, ensure header uses are dominated by preheader/dispatch values.

Types and Bridges
- Box handle is `i64` across NyRT boundary; strings prefer `i8*` fast paths.
- Convert rules: `ensure_i64/ensure_i1/ensure_ptr` style helpers (planned extraction) to centralize casting.

Harness (optional)
- llvmlite harness exists for fast prototyping and structural checks.
- Gate: `NYASH_LLVM_USE_HARNESS=1` (planned wiring); target parity tested by Acceptance A5.

References
- LOWERING_LLVM.md — lowering rules and runtime calls
- RESOLVER_API.md — Resolver design and usage
- LLVM_HARNESS.md — llvmlite harness interface and usage

