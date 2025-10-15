Selfhost Dev Drivers (quick helpers)

Scope
- Small Nyash programs under `apps/dev/` to exercise the selfhost compiler wrapper in minimal modes.
- They print a single JSON body (MIR JSON v0) using the existing compiler wrapper; execution remains in the parent runner.

Drivers
- `apps/dev/selfhost_compiler_min_binop.nyash`
  - Emits MIR(JSON v0) for Return(BinOp Add) in min-json mode + emit-mir.
- `apps/dev/selfhost_compiler_min_cmp.nyash`
  - Emits MIR(JSON v0) for Return(Compare Gt) in min-json mode + emit-mir.

How to run (Rust VM; quiet JSON-only)
```bash
NYASH_JSON_ONLY=1 ./target/release/nyash --backend vm apps/dev/selfhost_compiler_min_binop.nyash -- --dev
NYASH_JSON_ONLY=1 ./target/release/nyash --backend vm apps/dev/selfhost_compiler_min_cmp.nyash -- --dev
```

Notes
- These are dev-only helpers; no new defaults are introduced.
- For pipeline v2 emit-only flow, prefer `apps/selfhost-compiler/` boxes when present on the branch.

