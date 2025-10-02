# Self‑Hosting Quickstart (Phase 15 — Resume)

This note shows how to run the hakorune self‑host compiler MVP to emit AST JSON / MIR(JSON v0) and execute it with the current VM line. The flow keeps defaults unchanged and uses small, opt‑in flags for development.

## Layout
- Compiler MVP: `apps/selfhost-compiler/compiler.hako`
- Runtime helpers (dev): `apps/selfhost-runtime/`
- Mini‑VM samples (dev): `apps/selfhost/vm/`

## Run the self‑host compiler（Official runner path）
Use the runner’s selfhost pipeline with parent→child ENV forwarding. Defaults stay unchanged; all flags are opt‑in.

Examples (safe, short, quiet):
```
# Emit minimal AST JSON via pipeline v2 (header must contain {"version", "kind"})
NYASH_DISABLE_PLUGINS=1 \
NYASH_USE_NY_COMPILER=1 \
NYASH_NY_COMPILER_MIN_JSON=1 \
NYASH_NY_COMPILER_CHILD_ARGS="--pipeline-v2" \
NYASH_NY_COMPILER_EMIT_ONLY=1 \
NYASH_NY_COMPILER_SKIP_PY=1 \
NYASH_JSON_ONLY=1 \
timeout 5 ./target/release/hakorune --backend vm apps/examples/string_p0.hako

# Emit minimal MIR(JSON v0) (const→ret)
NYASH_DISABLE_PLUGINS=1 \
NYASH_USE_NY_COMPILER=1 \
NYASH_NY_COMPILER_MIN_JSON=1 \
NYASH_NY_COMPILER_CHILD_ARGS="--emit-mir" \
NYASH_NY_COMPILER_EMIT_ONLY=1 \
NYASH_NY_COMPILER_SKIP_PY=1 \
NYASH_JSON_ONLY=1 \
timeout 5 ./target/release/hakorune --backend vm apps/examples/string_p0.hako
```

Parent→child ENV mapping（official）
- `NYASH_NY_COMPILER_MIN_JSON=1` → child gets `-- --min-json`
- `NYASH_SELFHOST_READ_TMP=1`    → child gets `-- --read-tmp` (reads `tmp/ny_parser_input.ny`)
- `NYASH_NY_COMPILER_STAGE3=1`   → child gets `-- --stage3`
- `NYASH_NY_COMPILER_CHILD_ARGS="..."` → child gets extra args verbatim
- `NYASH_EMIT_TRACE=1`           → child gets `-- --emit-trace` (dev trace: 1行だけ [emit] 出力。最後のJSON行は不変)
- `NYASH_PREFER_CFG=1|NYASH_PREFER_CFG2=1` → child gets `-- --prefer-cfg` or `--prefer-cfg2`（CFG優先/材化あり）
- Timeouts / quiet pipe:
  - `NYASH_NY_COMPILER_TIMEOUT_MS=2000`（default）
  - `NYASH_JSON_ONLY=1`（suppress logs, print JSON only）

Direct run (dev only; requires allowing file using):
```
timeout 5 \
  NYASH_DISABLE_PLUGINS=1 NYASH_ENABLE_USING=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING_AST=1 NYASH_JSON_ONLY=1 \
  ./target/release/hakorune --backend vm apps/selfhost-compiler/compiler.hako -- --min-json
  
# Optional: pipeline v2 (emit-only)
timeout 5 \
  NYASH_DISABLE_PLUGINS=1 NYASH_USING=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING_STRATEGY=prelude NYASH_JSON_ONLY=1 \
  ./target/release/hakorune --backend vm apps/selfhost-compiler/compiler.hako -- --min-json --pipeline-v2
```

## Execute MIR(JSON v0)
Use the VM line (Rust) or PyVM harness as needed.

Rust VM (default):
```
./target/release/hakorune --backend vm apps/examples/json_query/main.hako
```

PyVM reference (when verifying parity):
```
NYASH_VM_USE_PY=1 ./target/release/hakorune --backend vm apps/examples/json_query/main.hako
```

LLVM harness (llvmlite):
```
NYASH_LLVM_USE_HARNESS=1 ./target/release/hakorune --backend llvm apps/examples/json_query/main.hako
```

Notes:
- For self‑host emitted JSON, route the file to your runner pipeline or a small loader script (dev only). Keep defaults unchanged in CI (no new jobs required).

## One‑shot dev smoke
Run quick selfhost checks (JSON header, normalization shapes, VM/LLVM parity when available):

```
tools/smokes/v2/run.sh --profile quick --filter '*selfhost*'
```

Notes on normalization
- Emitter delegates to `JsonProgramBox` which ensures stable shapes for Return/If/Loop/Call/Compare/Logical and always appends `meta.usings`.

## Flags (dev)
- Known rewrite default ON (userbox only, strict guards): `NYASH_REWRITE_KNOWN_DEFAULT=0|1`
- Router trace: `NYASH_ROUTER_TRACE=1`
- KPI sampling: `NYASH_DEBUG_KPI_KNOWN=1` (+ `NYASH_DEBUG_SAMPLE_EVERY=N`)
- Local SSA trace: `NYASH_LOCAL_SSA_TRACE=1`

## Acceptance (P6 resume)
- quick/integration remain green.
- Minimal self‑host emit→execute path PASS in a dev job (no CI change).
- No default behavior changes; all diagnostics under env flags.
