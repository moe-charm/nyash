Selfhost Compiler (Compiler Track)

Scope
- Nyash-written compiler MVP that emits Stage‑1 JSON and a minimal MIR(JSON v0).
- Lives entirely under `apps/selfhost-compiler/` to keep Core stable.

Run (official runner path)
- Minimal AST JSON (header):
  - `NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_MIN_JSON=1 NYASH_JSON_ONLY=1 timeout 5 ./target/release/nyash --backend vm apps/examples/string_p0.nyash`
- Minimal MIR(JSON v0) (const→ret / cmp→branch+jump→ret):
  - `NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_MIN_JSON=1 NYASH_NY_COMPILER_CHILD_ARGS="--emit-mir" NYASH_JSON_ONLY=1 timeout 5 ./target/release/nyash --backend vm apps/examples/string_p0.nyash`

Direct run (dev only)
- Allow file using and AST merge when running the Ny compiler program directly:
  - `NYASH_ENABLE_USING=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING_AST=1 ./target/release/nyash apps/selfhost-compiler/compiler.nyash -- --min-json`

Flags
- `NYASH_COMPILER_TRACK=1` — enable new builder/ssa/rewrite steps as they land (default OFF).
- `NYASH_JSON_ONLY=1` — print only JSON bodies (quiet acceptance runs).
- `DEV_TIMEOUT_SEC=60` — dev wrappers use timeout to avoid hangs.

Acceptance (dev)
- AST JSON: header must contain `{"version":..., "kind":...}` (non-empty).
- MIR JSON v0: contains a single main function.
  - Return(Int): single block `const`→`ret`.
  - Return(Compare(Int,Int)): 4 blocks CFG: entry(`const`×2→`compare`→`branch`), then(`const 1`→`jump`), else(`const 0`→`jump`), merge(`ret`).

Notes
- Core (`src/`) remains stable; compiler changes are scoped here and guarded by flags.
- Quick/integration profiles must remain green; compiler acceptance is dev-gated initially.
 - LocalSSA (builder/ssa/local.nyash): for MIR input, ensures `branch(cond)` has an in-block materialized value by injecting a `copy` at the block head when needed (behavior-preserving).

LocalSSA Trace (dev only)
- Default OFF. Enable from Ny code:
  - `using "apps/selfhost-compiler/builder/ssa/local.nyash" as LocalSSA`
  - `LocalSSA.trace_enable(1)`; then call `LocalSSA.ensure_cond(mir_json)`
  - Read stats via `LocalSSA.trace_get_map()` or `LocalSSA.trace_summary()`
- Selfhost CLI toggle (emit MIR path only): add `--ssa-trace` together with `--compiler-track --emit-mir`.
  - Trace values are not printed automatically (avoid polluting JSON output).
  - For ad-hoc checks, write a small Ny driver and print `LocalSSA.trace_summary()`.
