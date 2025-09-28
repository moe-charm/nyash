Selfhost Compiler (Compiler Track)

Scope
- Nyash-written compiler MVP that emits Stage‑1 JSON and a minimal MIR(JSON v0).
- Lives entirely under `apps/selfhost-compiler/` to keep Core stable.

Run (dev)
- Minimal AST JSON (safe path):
  - `NYASH_DISABLE_PLUGINS=1 NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1 NYASH_ALLOW_USING_FILE=1 NYASH_ENABLE_USING=1 ./target/release/nyash --backend vm apps/selfhost-compiler/compiler.nyash -- --min-json`
- Minimal MIR(JSON v0) (const→ret):
  - `... -- --min-json --emit-mir`

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

