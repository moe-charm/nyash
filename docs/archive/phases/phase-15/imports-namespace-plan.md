# Phase 15.3 — Imports/Namespace/nyash.toml Plan

Status: 15.3 planning; focus remains on Stage‑1/2 compiler MVP. This document scopes when and how to bring `nyash.toml`/include/import/namespace into the selfhost path without destabilizing parity.

Goals
- Keep runner‑level `nyash.toml` parsing/resolution as the source of truth during 15.3.
- Accept `using/import` syntax in the Ny compiler as a no‑op (record only) until resolution is delegated.
- Avoid VM changes; resolution happens before codegen.

Scope & Sequence (Phase 15.3)
1) Stage‑1/2 compiler stability (primary)
   - Ny→JSON v0 → Bridge → PyVM/llvmlite parity maintained
   - PHI merge remains on Bridge (If/Loop)
2) Imports/Namespace minimal acceptance (15.3‑late)
   - Parse `using ns` / `using "path" [as alias]` as statements in the Ny compiler
   - Do not resolve; emit no JSON entries (or emit metadata) — runner continues to strip/handle
   - Gate via `NYASH_ENABLE_USING=1`
3) Runner remains in charge
   - Keep existing Rust runner’s `using` stripping + modules registry population
   - `nyash.toml` parsing stays in Rust (Phase 15)

Out of scope (Phase 15)
- Porting `nyash.toml` parsing to Ny
- Cross‑module codegen/linking in Ny compiler
- Advanced include resolution / package graph

Acceptance (15.3)
- Ny compiler can lex/parse `using` forms without breaking Stage‑1/2 programs
- Runner path (Rust) continues to resolve `using` and `nyash.toml` as before (parity unchanged)

Looking ahead (MIR18 / Phase 16)
- Evaluate moving `nyash.toml` parsing to Ny as a library box (ConfigBox)
- Unify include/import/namespace into a single resolver pass in Ny with a small JSON side channel back to the runner
- Keep VM unchanged; all resolution before MIR build

Switches
- `NYASH_ENABLE_USING=1` — enable `using` acceptance in Ny compiler (no resolution)
- `NYASH_SKIP_TOML_ENV=1` — skip applying [env] in nyash.toml (existing)
