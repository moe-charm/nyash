Selfhost Compiler — Builder Layer (Scaffold)

Purpose
- Host compiler-local normalization passes that make emitted JSON more deterministic and VM-friendly without changing user-visible semantics.
- Keep Core (src/) stable; changes here are guarded and only affect the selfhost compiler app.

Boundaries
- Input: Stage‑1 JSON (from ParserBox.parse_program2).
- Output: Stage‑1 JSON (normalized) or minimal MIR(JSON v0) helpers (separately in mir_emitter_box).
- No file I/O; pure transformations only.

Modules (scaffold)
- ssa/local.nyash — LocalSSA materialization helpers (recv/arg/cond/cmp) within a basic block.
- ssa/loopssa.nyash — Loop/merge helpers (future): minimal PHI/var-map stabilization around headers/exits.
- rewrite/special.nyash — Early normalization (e.g., str/equals) keeping behavior unchanged.
- rewrite/known.nyash — Instance→Function rewrite when unique and safe (guarded; default OFF).

Guard / Flags
- NYASH_COMPILER_TRACK=1 — enable calling these passes from the compiler main (default OFF).

Design Notes
- Stage‑1 JSON schema remains the same; passes should be idempotent and fail-fast on malformed inputs.
- Start with no-op implementations; wire incrementally under flag to avoid regressions.

