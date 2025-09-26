Nyash GC Modes — Design and Usage

Overview
- Nyash adopts a pragmatic GC strategy that balances safety, performance, and simplicity.
- Default is reference counting with a periodic cycle collector; advanced modes exist for tuning and debugging.

User‑Facing Modes (recommended)
- rc+cycle (default, safe)
  - Reference counting with periodic cycle detection/collection.
  - Recommended for most applications; memory leaks from cycles are handled.
- minorgen (high‑performance)
  - Lightweight generational GC: moving nursery (Gen‑0), non‑moving upper generations.
  - Write barrier (old→new) is minimal; plugin/FFI objects remain non‑moving via handle indirection.

Advanced Modes (for language dev/debug)
- stw (debug/verification)
  - Non‑moving stop‑the‑world mark‑and‑sweep. Useful for strict correctness checks and leak cause isolation.
- rc (baseline for comparisons)
  - rc+cycle with cycle detection disabled. For performance comparisons or targeted debugging.
- off (expert, self‑responsibility)
  - Cycle detection and tracing off. Use only when cycles are guaranteed not to occur. Not recommended for long‑running services.

Selection & Precedence
- CLI: `--gc {auto,rc+cycle,minorgen,stw,rc,off}` (auto = rc+cycle)
- ENV: `NYASH_GC_MODE` (overridden by CLI)
- nyash.toml [env] applies last

Instrumentation & Diagnostics
- `NYASH_GC_METRICS=1`: print brief metrics (allocs/bytes/cycles/pauses)
- `NYASH_GC_METRICS_JSON=1`: emit JSON metrics for CI/aggregation
- `NYASH_GC_LEAK_DIAG=1`: on exit, dump suspected unreleased objects (Top‑K by type/site)
- `NYASH_GC_ALLOC_THRESHOLD=<N>`: warn or fail when allocations/bytes exceed threshold

Operational Guidance
- Default: rc+cycle for stable operations.
- Try minorgen when throughput/latency matter; it will fall back to rc+cycle on unsupported platforms or when plugin objects are declared non‑moving.
- off/rc are for special cases only; prefer enabling leak diagnostics when using them in development.

Implementation Roadmap (Step‑wise)
1) Wiring & Observability
   - Introduce `GcMode`, `GcController`, unify roots (handles, globals, frames) and safepoints.
   - Add `LeakRegistry` (allocation ledger) and exit‑time dump.
   - Ship rc+cycle (trial deletion) behind the controller (dev default can be rc+cycle).
2) minorgen (nursery)
   - Moving Gen‑0 with simple promotion; upper generations non‑moving mark‑sweep.
   - Minimal write barrier (old→new card marking). Plugin/FFI remain non‑moving.
3) stw (dev verify)
   - Non‑moving STW mark‑and‑sweep for correctness checks.

Notes
- Safepoint and barrier MIR ops already exist and are reused as GC coordination hooks.
- Handle indirection keeps future moving GCs compatible with plugin/FFI boundaries.

LLVM Safepoints
- Automatic safepoint insertion can be toggled for the LLVM harness/backend:
  - NYASH_LLVM_AUTO_SAFEPOINT=1 enables insertion (default 1)
  - Injection points: loop headers, function calls, externcalls, and selected boxcalls.
  - Safepoints call ny_check_safepoint/ny_safepoint in NyRT, which forwards to runtime hooks (GC.safepoint + scheduler poll).

Controller & Metrics
- The unified GcController implements GcHooks and aggregates metrics (safepoints/read/write/alloc).
- CountingGc is a thin wrapper around GcController for compatibility.
