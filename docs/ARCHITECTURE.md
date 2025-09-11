# Nyash Architecture (Phase 15)

## Scope and Priorities
- Primary execution path: LLVM AOT only. VM, Cranelift JIT/AOT, and the interpreter are not MIR14‑ready and are considered experimental in this phase.
- Minimize fallback logic. Prefer simple, predictable lowering over clever heuristics that diverge across backends.

## Value Model
- Box = handle (i64) as the canonical runtime representation.
- Strings: LLVM AOT favors i8* for fast path operations and bridging with NyRT. Conversions between i8* and handle exist but are kept to the minimum required surfaces.

## Division of Responsibilities
- NyRT (core, built‑in): fundamental boxes and operations essential for bootstrapping/self‑hosting.
  - IntegerBox, StringBox, ArrayBox, MapBox, BoolBox
  - Implemented as NyRT intrinsics (by‑id shims exist for plugin ABI compatibility).
- Plugins: external or platform‑dependent functionality (File/Net/Regex/HTTP/DB/GUI etc.).
- ExternCall: minimal window to the outside world (console print/log/error, debug trace, exit/now/readline); other APIs should route through BoxCall.

## Backend Policy (Phase 15)
- LLVM is the source of truth. All new rules and ABIs are documented for LLVM. Other backends will adopt them after LLVM stabilizes.
- Fallback logic must be narrow and documented. If behavior depends on type annotations, the (missing) annotations should be fixed at the MIR stage rather than widening fallback.

