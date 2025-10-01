Selfhost Compiler (Compiler Track)

Scope
- Nyash-written compiler MVP that emits Stage‑1 JSON and a minimal MIR(JSON v0).
- Lives entirely under `apps/selfhost-compiler/` to keep Core stable.

Run (official runner path)
- Minimal AST JSON (header):
  - `NYASH_DISABLE_PLUGINS=1 NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_MIN_JSON=1 NYASH_NY_COMPILER_EMIT_ONLY=1 NYASH_NY_COMPILER_SKIP_PY=1 NYASH_JSON_ONLY=1 timeout 5 ./target/release/nyash --backend vm apps/examples/string_p0.hako`
- Minimal MIR(JSON v0) (const→ret / cmp→branch+jump→ret):
  - `NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_MIN_JSON=1 NYASH_NY_COMPILER_CHILD_ARGS="--emit-mir" NYASH_JSON_ONLY=1 timeout 5 ./target/release/nyash --backend vm apps/examples/string_p0.hako`

Execution policy (Phase 15.7)
- Default executor: Rust VM. PyVM is used only when explicitly requested by env.
  - Prefer Rust VM: do not set `NYASH_VM_USE_PY` (default). Runner executes MIR on Rust VM.
  - Use PyVM (opt‑in): set `NYASH_VM_USE_PY=1` to execute via the Python harness for parity checks.
  - Parser harness (Python) for JSON v0 emission can be skipped via `NYASH_NY_COMPILER_SKIP_PY=1` if you want a fully Rust‑only path.

Direct run (dev only)
- Allow file using and AST merge when running the Ny compiler program directly (use timeout to prevent hangs):
  - `timeout 5s NYASH_DISABLE_PLUGINS=1 NYASH_ENABLE_USING=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING_AST=1 NYASH_JSON_ONLY=1 ./target/release/nyash --backend vm apps/selfhost-compiler/compiler.hako -- --min-json`
- Optional (emit-only pipeline v2): append `--pipeline-v2` to route via ExecutionPipelineBox
  - `... apps/selfhost-compiler/compiler.hako -- --min-json --pipeline-v2`
 - Optional (builder bridge to PipelineV2 under emit-mir): append `--emit-mir --builder-bridge` and prefer CFG level
   - `... apps/selfhost-compiler/compiler.hako -- --min-json --emit-mir --builder-bridge --prefer-cfg2`
   - prefer levels: `--prefer-cfg` (CFG/no copy), `--prefer-cfg2` (CFG/materialize copy)

Flags
- `NYASH_COMPILER_TRACK=1` — enable new builder/ssa/rewrite steps as they land (default OFF).
- `NYASH_JSON_ONLY=1` — print only JSON bodies (quiet acceptance runs).
- `DEV_TIMEOUT_SEC=60` — dev wrappers use timeout to avoid hangs.
Environment (selfhost pipeline)
- `NYASH_USE_NY_COMPILER=1` — enable Ny compiler child path (emit‑only by default).
- `NYASH_NY_COMPILER_EMIT_ONLY=1` — emit JSON to stdout and stop (default).
- `NYASH_NY_COMPILER_MIN_JSON=1` — pass `-- --min-json` to child; header must be non‑empty.
- `NYASH_NY_COMPILER_CHILD_ARGS` — extra args to child (e.g., `--emit-mir`).
- `NYASH_NY_COMPILER_TIMEOUT_MS=2000` — child timeout (ms), fail‑fast on violation.
- `NYASH_NY_COMPILER_USE_TMP_ONLY=1` — reuse existing tmp input file (dev only).
- `NYASH_NY_COMPILER_SKIP_PY=1` — skip Python MVP harness stage entirely.
- `NYASH_VM_USE_PY=1` — execute MIR via PyVM (otherwise Rust VM is used).
- `NYASH_ENABLE_USING=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING_AST=1` — allow using/AST prelude when running Ny compiler directly.

Acceptance (dev)
- AST JSON: header must contain `{"version":..., "kind":...}` (non-empty).
- MIR JSON v0: contains a single main function.
  - Return(Int): single block `const`→`ret`.
  - Return(Compare(Int,Int)): 4 blocks CFG: entry(`const`×2→`compare`→`branch`), then(`const 1`→`jump`), else(`const 0`→`jump`), merge(`ret`).

Notes
- Core (`src/`) remains stable; compiler changes are scoped here and guarded by flags.
- Quick/integration profiles must remain green; compiler acceptance is dev-gated initially.
 - LocalSSA (builder/ssa/local.nyash): for MIR input, ensures `branch(cond)` has an in-block materialized value by injecting a `copy` at the block head when needed (behavior-preserving).

Failure policy (Fail‑Fast)
- No silent fallbacks. Timeouts, parse errors, or malformed JSON v0 must terminate with a clear diagnostic.
- When `NYASH_JSON_ONLY=1`, only JSON lines are printed; diagnostics go to stderr. The runtime honors quiet mode and suppresses non-essential init logs.

Planned smokes (quick / lightweight)
- selfhost_json_guard: `--min-json` header is non‑empty (emit‑only). Rust VM default.
- selfhost_min_mir_const_ret: minimal MIR(JSON v0) const→ret executes with Rust VM.
- selfhost_compare_branch_parity: compare→branch→ret parity check (Rust VM; PyVM optional).

LocalSSA Trace (dev only)
- Default OFF. Enable from Ny code:
  - `using "apps/selfhost-compiler/builder/ssa/local.nyash" as LocalSSA`
  - `LocalSSA.trace_enable(1)`; then call `LocalSSA.ensure_cond(mir_json)`
  - Read stats via `LocalSSA.trace_get_map()` or `LocalSSA.trace_summary()`
- Selfhost CLI toggle (emit MIR path only): add `--ssa-trace` together with `--compiler-track --emit-mir`.
  - Trace values are not printed automatically (avoid polluting JSON output).
  - For ad-hoc checks, write a small Ny driver and print `LocalSSA.trace_summary()`.
Pipeline v2 (Box-First, emit-only)
- **📋 詳細設計**: [docs/development/selfhosting/pipeline_v2.md](../../docs/development/selfhosting/pipeline_v2.md)
- **📦 実装ガイド**: [pipeline_v2/README.md](pipeline_v2/README.md)
- **🔧 インターフェース契約**: [INTERFACES.md](INTERFACES.md)
- Skeleton boxes live under `pipeline_v2/`:
  - `ExecutionPipelineBox`: orchestrates ParserBox→EmitterBox; prints one JSON line.
  - `BackendBox` (stub): backend tag only; no execution in Phase 15.7.
  - `MirBuilderBox` (stub): reserved for Ny→MIR lowering.
- Example (dev):
  - `using "apps/selfhost-compiler/pipeline_v2/execution_pipeline_box.hako" as Px`
  - `local p = new Px()`; `p.run_source("return 0", 0)`
