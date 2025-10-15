# Frontend Layers — Parser / Resolver (Contracts)

Purpose
- Define clear boundaries between parsing and resolving.
- Keep responsibilities crisp and testable, enabling phased migration to Hakorune.

Dual Parser Mode (contract)
- Minimal JSON v0 header (both parsers output the same minimal shape):
  - `{"version":"0","kind":"Program","stats":{"stmts":<int>}}`
  - Additional fields are allowed but ignored in cross‑check smokes.
- Keys compared in smokes (both mode): `version`, `kind`, `stats.stmts`.
- CLI flags (smokes harness):
  - `SMOKES_PARSER_MODE=rust|hako|both` (default `rust`)
  - `rust`: parse via Rust facade; `hako`: run selfhost compiler (`--min-json`); `both`: run both and compare minimal keys.

Contracts
- Parser
  - Input: tokens (from `tokenizer`)
  - Output: AST-like structure implementing `layers::ParserOutput`
  - Forbidden: name resolution, code generation, runtime behavior
- Resolver
  - Input: `layers::ResolverInput` (extends `ParserOutput`)
  - Output: resolved program (names/imports/types) — format TBD
  - Forbidden: code generation, runtime behavior

Rust Side (current step)
- Traits added in `src/layers/interfaces.rs`.
- Guards added:
  - `src/front/parser_layer/` and `src/front/resolver_layer/` (docs + LAYER_GUARD only)
- Implementations remain in existing modules until migration is complete.

How to verify (smokes)
- Parser minimal JSON header (selfhost child):
  - `SMOKES_SELFHOST_ENABLE=1 tools/smokes/v2/run.sh --profile quick-selfhost --filter selfhost_min_json_header_vm.sh`
 - Parser facade path (opt-in):
   - `HAKO_FRONT_USE_FACADE=1 tools/smokes/v2/run.sh --profile quick-selfhost --filter parser_facade_min_vm.sh`

Notes
- This document describes contracts only; no default behavior is changed by adding these files.
- When extending capabilities, update the traits first, then add docs/tests, then patch code.
 - Dev flag (opt-in) to route parsing via facade: `HAKO_FRONT_USE_FACADE=1` (default OFF)
