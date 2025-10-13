# Phase‑15.7 — Async/Sync Lowering Plan (Short)

Scope
- Stabilize `callAsync` → scheduler → `FutureBox` across backends.
- Keep LLVM path green via ny‑llvmc (crate) and llvmlite harness for minimal async.

Milestones
1) Docs first (this file; guides/async-model.md; llvm‑hako‑abi async notes) — done
2) Smokes
   - quick‑selfhost: callable_async, callable_async_parallel — green
   - curated_llvm: async‑await‑min — keep green; spawn -> SKIP until builder fix
3) Builder fixes (llvmlite)
   - Implement `_resolve_arg` in Python builder to support spawn/instance
   - Add timeouts for harness subprocess; Fail‑Fast on missing `.o`
4) Batch timer harness
   - Reduce TimerBox checks (batching+calibration) in fixed‑time suite
   - Optional async‑timer flag for stop‑flag scheduling
5) Parity sweep
   - Rust VM ↔ Hakorune‑VM ↔ LLVM ops/sec within target bands on fixed‑time suite

Policies
- Sync default; async opt‑in only. No silent fallbacks.
- Determinism mode denies external IO/threads (futures must be pure compute or error).
