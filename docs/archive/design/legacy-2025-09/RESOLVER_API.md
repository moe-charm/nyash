# Resolver API (Minimal i64 Prototype)

Goals
- Phase‑15（MIR13運用）における方針: MIR 生成層は PHI を出さず、LLVM 層で PHI を合成する。
- Centralize "ValueId → current-block value" resolution.
- Guarantee dominance by localizing values at the start of the block (before non-PHI).
- De-duplicate per (block, value) to avoid redundant PHIs/casts.

Design
- Resolver-only reads: lowerers must fetch cross-block values via `Resolver` (ban direct `vmap.get` for reads across BBs).
- `Resolver` keeps small per-function caches keyed by `(BasicBlockId, ValueId)`.
- `resolve_i64(...)` returns an `i64`-typed value, inserting a PHI at the beginning of the current block and wiring
  incoming from predecessor end-snapshots (sealed SSA). Any required casts are inserted in predecessor blocks just
  before their terminators to preserve dominance.
- `resolve_ptr(...)` returns an `i8*` value localized to the current block; integer handles are bridged via `inttoptr`.
- `resolve_f64(...)` returns an `f64` value localized to the current block; ints bridged via `sitofp`.
- Internally uses sealed snapshots (`block_end_values`) to avoid referencing values that do not dominate the use.

Usage (planned wiring)
- Create `let mut resolver = instructions::Resolver::new();` at function lowering start.
- Replace all integer value fetches in lowerers with `resolver.resolve_i64(...)`.
- Keep builder insertion discipline via `BuilderCursor`.

Ban: Direct `vmap.get(..)` for cross-BB reads
- Direct reads from `vmap` are allowed only for values defined in the same block (after they are created).
- For any value that may come from a predecessor, always go through `Resolver`.
- CI guard: keep `rg "vmap\.get\(" src/backend/llvm` at zero for instruction paths (Resolver-only).

Next
- Migrate remaining `localize_to_i64` call sites to the resolver.
- Enforce vmap direct access ban in lowerers (Resolver-only for reads).
- ループ（while 形 CFG）の検出と、ヘッダ BB での搬送 PHI 合成（preheader/backedge の 2 incoming）を実装。

Tracing
- `NYASH_LLVM_TRACE_PHI=1`: log PHI creation/wiring in the Rust/inkwell path.
- `NYASH_LLVM_TRACE_FINAL=1`: in the Python/llvmlite harness, trace selected final calls (e.g., `Main.node_json/3`,
  `Main.esc_json/1`) to correlate ON/OFF outputs during parity checks.

Acceptance tie-in
- Combined with LoopForm: dispatch-only PHI + resolver-based value access → dominance violations drop to zero (A2.5).
