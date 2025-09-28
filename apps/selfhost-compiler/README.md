Selfhost Compiler (Compiler Track)

Scope
- Nyash-written compiler MVP that emits Stage‑1 JSON and a minimal MIR(JSON v0).
- Lives entirely under `apps/selfhost-compiler/` to keep Core stable.

Run (official runner path)
- Minimal AST JSON (header):
  - `NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_MIN_JSON=1 NYASH_JSON_ONLY=1 timeout 5 ./target/release/nyash --backend vm apps/examples/string_p0.nyash`
- Minimal MIR(JSON v0) (const→ret):
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
- MIR JSON v0: must contain a single main function with a block of `const` then `ret`.

Notes
- Core (`src/`) remains stable; compiler changes are scoped here and guarded by flags.
- Quick/integration profiles must remain green; compiler acceptance is dev-gated initially.
