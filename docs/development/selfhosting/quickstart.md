# Self‑Hosting Quickstart (Phase 15 — Resume)

This note shows how to run the Nyash self‑host compiler MVP to emit MIR(JSON v0) and execute it with the current VM line. The flow keeps defaults unchanged and uses small, opt‑in flags for development.

## Layout
- Compiler MVP: `apps/selfhost-compiler/compiler.nyash`
- Runtime helpers (dev): `apps/selfhost-runtime/`
- Mini‑VM samples (dev): `apps/selfhost/vm/`

## Run the self‑host compiler
Compile a minimal program (string embedded in the compiler) and print JSON:

```
./target/release/nyash apps/selfhost-compiler/compiler.nyash -- --stage3
```

ENV → child args (透過):
- `NYASH_NY_COMPILER_MIN_JSON=1` → `-- --min-json`
- `NYASH_SELFHOST_READ_TMP=1`    → `-- --read-tmp` (reads `tmp/ny_parser_input.ny`)
- `NYASH_NY_COMPILER_STAGE3=1`   → `-- --stage3` (Stage‑3 surface enable)
- `NYASH_NY_COMPILER_CHILD_ARGS="..."` → passes extra args verbatim

Examples:
```
NYASH_NY_COMPILER_MIN_JSON=1 ./target/release/nyash apps/selfhost-compiler/compiler.nyash -- --stage3 > /tmp/out.json
NYASH_SELFHOST_READ_TMP=1 ./target/release/nyash apps/selfhost-compiler/compiler.nyash -- --min-json --stage3
```

## Execute MIR(JSON v0)
Use the VM line (Rust) or PyVM harness as needed.

Rust VM (default):
```
./target/release/nyash --backend vm apps/examples/json_query/main.nyash
```

PyVM reference (when verifying parity):
```
NYASH_VM_USE_PY=1 ./target/release/nyash --backend vm apps/examples/json_query/main.nyash
```

LLVM harness (llvmlite):
```
NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm apps/examples/json_query/main.nyash
```

Notes:
- For self‑host emitted JSON, route the file to your runner pipeline or a small loader script (dev only). Keep defaults unchanged in CI (no new jobs required).

## One‑shot dev smoke
Run a minimal end‑to‑end smoke that tries to emit JSON (best‑effort) and verifies VM outputs match with Known rewrite ON/OFF:

```
tools/selfhost_smoke.sh
```

It does not modify defaults and is safe to run locally.

## Flags (dev)
- Known rewrite default ON (userbox only, strict guards): `NYASH_REWRITE_KNOWN_DEFAULT=0|1`
- Router trace: `NYASH_ROUTER_TRACE=1`
- KPI sampling: `NYASH_DEBUG_KPI_KNOWN=1` (+ `NYASH_DEBUG_SAMPLE_EVERY=N`)
- Local SSA trace: `NYASH_LOCAL_SSA_TRACE=1`

## Acceptance (P6 resume)
- quick/integration remain green.
- Minimal self‑host emit→execute path PASS in a dev job (no CI change).
- No default behavior changes; all diagnostics under env flags.
