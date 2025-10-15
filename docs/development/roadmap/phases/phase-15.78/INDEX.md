# Phase 15.78 — Frozen UX Polish, AOT Parity, Legacy Cleanup

Purpose
- Turn the frozen line into a “useful distribution”: smooth packaging, simple run, light parity checks, and safer legacy retreat.

Tracks
- A. Distribution & UX
  - Package dist/ as archives (Linux/Win-gnu/Win-msvc/All-in-one)
  - release.json enriched (support_matrix, sizes, sha256)
  - Doctor options: --no-build, --verbose; actionable hints
  - Quickstart README in archives
- B. AOT parity & ny_main glue
  - Simple glue generator (ny_main) via CLI/subcommand or helper script
  - Light parity smokes (String.len / Array.size / Map.size) VM↔AOT
- C. Legacy staged retreat
  - plugin-only profile: legacy-boxes default OFF (quick stays green)
  - Remove unreachable router arms (warning cleanup)
  - Track remaining crate::boxes refs and reduce
- D. Windows final touches
  - One-command guide for MSVC/MinGW quicklinks
  - Doctor hints for VS/clang PATH and common pitfalls

DoD
- dist/pkg/* archives + dist/release.json (support_matrix included)
- Frozen Quickstart in archives and docs
- 3–4 light parity smokes pass under VM/AOT (no external I/O)
- plugin-only profile stays green with legacy-boxes OFF; unreachable code pruned
- Windows MSVC/MinGW doc paths verified (no missing steps)

Notes
- Keep existing profiles green. Broader refactors are out of scope; focus on polish and small wins.

