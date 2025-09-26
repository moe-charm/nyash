# Phase‑15 Macro‑Driven Replan

Status: Adopted (2025‑09). This plan reframes Phase‑15 self‑hosting to leverage the new macro system (AST JSON v0 / PyVM sandbox / strict / timeout / golden).

## Goals
- Consolidate all front‑end, sugar, and light rewrites into user macros (Nyash/PyVM)
- Keep Rust core minimal: parse/AST emit/MIR build/backend only
- Deterministic expansion with strict failure and timeouts; golden testing as acceptance

## Execution Pipeline
1) Parse (Rust) → AST
2) Macro Expansion (fixed‑point, 1 pass): Built‑in (Rust) → User Macros (Nyash/PyVM)
3) Using/resolve → MIR → Backend (VM/LLVM/WASM/AOT)

## Guardrails
- strict=1 (default): child error/timeout aborts the build
- timeout: `NYASH_NY_COMPILER_TIMEOUT_MS=2000` (default)
- capabilities: all OFF (io/net/env=false) for Phase‑15
- observability: `--dump-expanded-ast-json` + JSONL trace (`NYASH_MACRO_TRACE_JSONL`)

## Work Items
1) Enable PyVM runner route for macros (done)
   - ランナールートが既定。内部子ルートは非推奨（`NYASH_MACRO_BOX_CHILD_RUNNER=0` でのみ強制）
2) Identity + upper_string templates (done)
   - Examples under `apps/macros/examples/`
3) Golden tests
   - identity/upper cases (done)
   - add 1–2 more (array/map literal touch) [next]
4) Selfhost compiler refactor (front)
   - Limit Ny parser to Stage‑2 subset; sugar via macros
   - Keep resolver in runner (Phase‑15 policy)
5) nyash.toml sketch for macro registration and caps
   - `[macros]` + `[macro_caps."path"]` io/net/env=false (docs only in Phase‑15)

## Acceptance
- Expanded AST JSON matches goldens for sample programs
- Macro runner path green under strict=1, timeout=2000ms
- MIR/LLVM/VM paths stable with expanded inputs
