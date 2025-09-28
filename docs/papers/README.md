Nyash Papers — Top-Level

Purpose
- Provide a clean, public-friendly structure for preparing papers about Nyash (language, VM, compiler, and build philosophy).
- Keep artifacts reproducible with small scripts and clear, spec-neutral claims.

Layout
- nyash-phase15.7-selfhost/ — Draft focusing on Phase 15.7 outcomes: Unified method resolution, Mini‑VM in Ny, and llvmlite harness parity.
- TEMPLATE.md — Minimal section scaffold you can copy for new drafts.

Rules
- Keep claims spec-neutral unless a flag is required; prefer dev-only observations gated by env.
- Every result must be runnable locally (scripts under tools/papers/).
- Prefer quick profile evidences first; integration for parity; avoid heavy or flaky tests.

