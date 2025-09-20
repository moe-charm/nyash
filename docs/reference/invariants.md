# Nyash Invariants (Spec)

This document lists non‑negotiable invariants the implementation preserves across backends. They’re used as a contract for testing and design.

Core
- PHI hygiene
  - No empty PHI nodes are emitted in LLVM IR (temporary safety valve exists behind `NYASH_LLVM_SANITIZE_EMPTY_PHI=1`).
  - All PHIs appear at the beginning of a basic block.
- Match/Peek
  - A match scrutinee is evaluated exactly once, bound to an internal gensym.
  - Guard conditions are logically conjoined with the arm predicate; fallthrough reaches the next arm.
- Exceptions
  - Exceptions follow “scope first” semantics. Postfix `catch/cleanup` normalize to a single `TryCatch` block around the immediately‑preceding expression.
- LoopForm
  - Loop bodies may be normalized where safe to a stable order: non‑assign statements then assign statements. No semantic reorder is performed when a non‑assign appears after any assign.
  - Carrier analysis emits observation hints only (zero runtime cost).
  - Break/continue lowering is unified via LoopBuilder; nested bare blocks inside loops are handled consistently (Program nodes recurse into loop‑aware lowering).
- Scope
  - Enter/Leave scope events are observable through MIR hints; they do not affect program semantics.

Observability
- MIR hints can be traced via `NYASH_MIR_HINTS` (pipe style): `trace|scope|join|loop|phi` or `jsonl=path|loop`.
- Golden/Smoke tests cover representative invariants.

Backends
- PyVM and LLVM share semantics; PyVM prioritizes clarity over performance.
- Cranelift/WASM routes inherit the same invariants when enabled.
