# Stage 4 — Dual Parser Harness (ChatGPT Minimal Plan)

Purpose
- Provide a streamlined, 2-day, minimal-but-sufficient Stage‑4 plan focused on a thin, stable C ABI harness and a simple integration path. Keeps Rust as a thin router and prepares Hakorune to call the same C ABI later.

What’s different vs original Stage‑4 docs
- Single thin C ABI: two functions only (parse_source_dual, free_parse_result)
- Minimal result header (stmt_count/kind + stable header with struct_size)
- Rust side exports exactly two externs (parse_source_rust, parse_source_hako)
- Feature-gated build (parser-c-abi) and a single smoke (both-mode header match)

Read Next
- QUICKSTART (2-day steps): ./QUICKSTART.md
- Minimal C ABI Spec: ./C_ABI_MIN_SPEC.md
- Integration guide (build.rs, Rust externs, runner hook): ./INTEGRATION.md
- Integration Strategy (beyond Stage‑4): ./INTEGRATION_STRATEGY_CLAUDE.md
- extern_c Self‑Host Strategy (dynamic FFI end‑game): ./EXTERN_C_SELFHOST_STRATEGY.md

Definition of Done (Stage‑4)
- C harness builds behind feature flag (parser-c-abi)
- Rust externs compile and return minimal header (rust path OK; hako path may be stubbed initially)
- Smoke (both-mode, header-only) passes: version/kind/stmts equal
- Rollback: feature OFF restores previous path with zero code removal

Out of Scope (Stage‑4)
- Full parser parity; advanced error mapping; JSON AST comparisons. Only minimal header parity is required.
