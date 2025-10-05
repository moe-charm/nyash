# Phase 15.9 — Constructor Contracts Optimization Plan

Scope
- Keep runtime constructor contracts (unborn/in_birth/born) always-on for correctness.
- Allow safe elision via optimization only when provably redundant.

Semantics (fixed)
- States: unborn → in_birth (during birth) → born (on success) / unborn (on failure).
- Guard at all instance-operation entrypoints: born || (in_birth && same_instance).
- Idempotence: second birth after success is no-op. Re-entrancy during birth is an error.

Optimization Strategy (post self-host)
- Insert runtime helpers in LLVM (nyrt):
  - contracts_birth_enter(handle)
  - contracts_birth_exit(handle, success)
  - contracts_check(handle, me_opt)
- Builder/VM invariants enable SSA-based proofs:
  - If basic block is strictly dominated by a successful birth for handle H, insert llvm.assume(handle_is_born(H)).
  - DCE: conditions in subsequent contracts_check fold to true and are removed.
  - Exceptional/alternate paths remain guarded (no semantic change).

Profiles
- dev/ci: checks always enabled; verbose trace gated by env.
- release-safe (default): enable local elimination where proofs hold; do not cross inlining boundaries.
- perf (opt-in): allow cross-BB/cross-Fn assumptions when annotated (future work).

Out of scope (for now)
- Full static arity/type schema for birth args (kept as future flag, off by default).
- JSON v0 bridge hard enable of auto_birth (kept behind env until stable).

Acceptance
- Quick + integration suites green with contracts on.
- Dedicated smokes for: auto-birth, unborn fail-fast, unborn→birth→ok, plugin no-birth, cross-module birth, birth re-entrancy error.

