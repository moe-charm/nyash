# Async Task System — Phased Plan (P1–P3)

## Phase 1: Foundations (stabilize + timeouts)

- FutureBox: switch to Mutex+Condvar (done).
- Await: poll scheduler + timeout gate in VM and JIT (done; unify to Result.Err in P3).
- env.future.spawn_instance: enqueue via Scheduler; fallback sync if no scheduler (done).
- Safepoint: ensure env.runtime.checkpoint is emitted around await (Lowerer rule). 
- Smokes: async-await-min; async-spawn-instance (safe form, no env).
- Acceptance:
  - No hangs (await respects timeout); CPU near idle while waiting.
  - VM/JIT pass basic smokes; lingering processes do not remain.

## Phase 2: TaskGroup & CancellationToken

- Types:
  - CancellationToken { cancel(), is_cancelled() } idempotent; parent→child propagation only.
  - TaskGroup Box { spawn(fn)->Future, cancelAll(), joinAll() }, owns token; fini enforces cancel→join.
- API:
  - nowait sugar targets current TaskGroup.
  - Unsafe detached spawn hidden behind unsafe_spawn_detached() and verifier checks.
- VM implementation:
  - Extend scheduler to accept token; tasks periodically check token or are cancelled at await.
  - Group registry per scope; insert fini hooks in function epilogues and Main.
- JIT/EXE:
  - NyRT shims accept optional token handle; if missing, use current group’s token.
- Smokes:
  - spawn_cancel_on_scope_exit; nested_groups; lifo_join_order.
- Acceptance:
  - Parent exit cancels and joins children deterministically (LIFO); no leaks per leak tracker.

## Phase 3: Error Semantics & Unification

- Future.await returns Result<T, Err> (Timeout/Cancelled/Panic) consistently (VM/JIT).
- Remove 0/None fallbacks; map shims to Result at Nyash layer.
- Lowerer verifies checkpoints around await; add verifier rule.
- Observability: minimal counters and optional traces.
- Smokes:
  - await_timeout distinct from cancel; panic_propagation; wakeup_race (no double resolve).
- Acceptance:
  - Consistent error surface; result handling identical across VM/JIT/EXE.

## Test Matrix & CI

- Backends: {vm, jit, aot} × Modes: {default, strict}.
- Smokes kept minimal; time‑bounded via timeout(1) wrapper.
- CPU spin test: ensure idle waiting; measured via time/ps (best‑effort).

## Migration & Compatibility

- Keep env.future.spawn_instance during transition; TaskGroup.spawn preferred.
- nowait sugar remains; mapped to TaskGroup.spawn.
- Document flags: NYASH_AWAIT_MAX_MS, NYASH_SCHED_*.

## Files & Ownership

- Spec & Plan live here; updates linked from CURRENT_TASK.md.
- Code changes limited to runtime/{scheduler,global_hooks}, boxes/future, jit/extern/async, lowerer await rules.

