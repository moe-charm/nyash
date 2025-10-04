# Nyash Phase 15.7 — Self-Host Compiler Path with Unified Calls and Mini‑VM

Key Claims (spec-neutral)
- Unified method resolution simplifies and stabilizes call emission; dev guards keep behavior unchanged by default.
- Mini‑VM written in Nyash (MirVmMin) executes a minimal MIR(JSON v0) subset (const/binop/compare/branch/jump/ret) and provides ground-truth checks for segmentation and control-flow invariants.
- llvmlite harness parity confirms correctness for AOT line while Rust VM remains primary runtime.

What We Did
- RouterPolicy + ReceiverInference: consistent String-like normalization and Unknown→BoxCall fallback (always-on, stability-first).
- LocalSSA + Materialize: per-block copies enforce φ→Copy→Call order; avoid use-before-def.
- VarMapGuard: prevent accidental me-binding at PHI merges.
- Mini‑VM: single-pass brace-balanced segmentation; new edge smokes (compare mix, 0-div/mod, no-ret fallback, branch undef cond, jump chain).

Results
- Quick profile: 72/72 PASS; Integration: PASS (llvmlite harness).
- Mini‑VM returns -1 with an [ERROR] line on undefined ret to keep fail‑fast visible and distinct from false(0).
- Self-host compiler (dev-only): JSON head non-empty via `NYASH_JSON_ONLY=1` gate.

Next
- Advance self-host compiler MVP; keep dev valves default-OFF; remove them once rewrite/inference are fully robust.

