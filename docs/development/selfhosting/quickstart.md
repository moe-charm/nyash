# Self‑Hosting Quickstart (Phase 15 — Resume)

This note shows how to run the Nyash self‑host compiler MVP to emit MIR(JSON v0) and execute it with the current VM line. The flow keeps defaults unchanged and uses small, opt‑in flags for development.

## Layout
- Compiler MVP: `apps/selfhost-compiler/compiler.nyash`
- Runtime helpers (dev): `apps/selfhost-runtime/`
- Mini‑VM samples (dev): `apps/selfhost/vm/`

## Run the self‑host compiler（Official runner path）
Use the runner’s selfhost pipeline with parent→child ENV forwarding. Defaults stay unchanged; all flags are opt‑in.

Examples (safe, short, quiet):
```
# Emit minimal AST JSON (header must contain {"version", "kind"})
NYASH_USE_NY_COMPILER=1 \
NYASH_NY_COMPILER_MIN_JSON=1 \
NYASH_JSON_ONLY=1 \
timeout 5 ./target/release/nyash --backend vm apps/examples/string_p0.nyash

# Emit minimal MIR(JSON v0) (const→ret)
NYASH_USE_NY_COMPILER=1 \
NYASH_NY_COMPILER_MIN_JSON=1 \
NYASH_NY_COMPILER_CHILD_ARGS="--emit-mir" \
NYASH_JSON_ONLY=1 \
timeout 5 ./target/release/nyash --backend vm apps/examples/string_p0.nyash
```

Parent→child ENV mapping（official）
- `NYASH_NY_COMPILER_MIN_JSON=1` → child gets `-- --min-json`
- `NYASH_SELFHOST_READ_TMP=1`    → child gets `-- --read-tmp` (reads `tmp/ny_parser_input.ny`)
- `NYASH_NY_COMPILER_STAGE3=1`   → child gets `-- --stage3`
- `NYASH_NY_COMPILER_CHILD_ARGS="..."` → child gets extra args verbatim
- Timeouts / quiet pipe:
  - `NYASH_NY_COMPILER_TIMEOUT_MS=2000`（default）
  - `NYASH_JSON_ONLY=1`（suppress logs, print JSON only）

Direct run (dev only; requires allowing file using):
```
NYASH_ENABLE_USING=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING_AST=1 \
  ./target/release/nyash apps/selfhost-compiler/compiler.nyash -- --min-json
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
