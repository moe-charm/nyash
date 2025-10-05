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



---

# Phase 15.9 — Optimization Track (Consolidated)

Status: active (optimization-focused, spec‑preserving)

Scope
- Consolidate the performance plan across Mini‑VM and AOT/LLVM while keeping public semantics unchanged.
- Introduce fast, boxed‑cost‑free paths for primitives and a thin FFI route for heavy work.

Goals
- VM: unboxed primitives for const/binop/compare/branch/jump/phi/ret.
- AOT: fast standalone executables; zero diagnostics/contract cost in release.
- Contracts: dev‑only visibility; release path is zero‑overhead.

Deliverables
- Acceleration DSL (flagged): `#[fast] { ... }` lowers to primitive MIR only (no Box paths).
- Minimal `externcall` to C ABI (i64/f64/bool) with fail‑fast validation and AOT compatibility.
- Dispatch caches: MonoIC → small PIC at hot call‑sites keyed by `<type,method>`.
- SBO for Array/Map (N≤4/8) and arena for short‑lived temporaries.

Language Surface (flagged; staged)
- `#[fast] { ... }`
  - Primitive returns: i64/f64/bool/void
  - Allowed: vars, arithmetic, compares, if/while/for, small arrays
  - Forbidden: Box creation, plugins, I/O, exceptions, dynamic dispatch
  - Violations: build‑time error (DEV diagnostics only)
- `externcall`
  - Decl: `extern "C" add(i64,i64)->i64`
  - Guard: signature match; else fail‑fast

Runtime Representation
- Tagged union for i64/f64/bool/null/void; BoxRefs elsewhere
- Contracts: in dev, record new/birth/in_birth (idempotent; reentrancy error). Release removes checks/logs.

Dispatch & Allocation
- MonoIC per site → small PIC (N≤4) on miss frequency
- SBO for small Array/Map; arena for short‑lived objects

Benchmarks & KPIs
- Microbenches: int loop, compare→branch diamond, multi‑φ, SmallArray ops, SmallMap ops
- Targets: VM 1.5–3.0x CPython; AOT 2–10x CPython on those kernels

Flags & Guards
- `NYASH_ACCEL_ENABLE=1` — enable fast blocks + FFI (dev‑only)
- `NYASH_DEV_JSON_MARKER=1` — Ny‑side diagnostics via `{"__dev__":1}`
- `NYASH_CHECK_CONTRACTS=1` — dev contracts (OFF in release)

Rollout (small, reversible)
1) Unboxed VM for compare/binop → ret
2) MonoIC (→ PIC) for method/box dispatch
3) SBO + arena for small & short‑lived allocations
4) `#[fast] {}` minimal + smokes (with `NYASH_ACCEL_ENABLE=1`)
5) `externcall` minimal + smokes (i64 add)

Risks & Mitigations
- Parser conflict: start with `#[fast] {}`; postpone special `[]` form
- ABI: restrict to i64/f64/bool; fail‑fast on mismatches; add smokes
- Complexity: isolate in boxes; no frozen Core rewrites

Validation
- Golden MIR: fast blocks emit only primitive ops
- Smokes: fast/externcall; contracts; plugin birth/no‑birth
- Perf scripts: CPython 3.11 comparison on 5 microbenches
