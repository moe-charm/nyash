# Phase 15.77 — Milestone (Frozen Toolchain Polish & Windows Plan)

Status: In‑Flight

Objectives
- Replace development stubs with static runtime on Windows (MinGW → MSVC parity)
- Finalize doctor & docs (end‑to‑end clarity, copy/paste quicklinks)

Deliverables (DoD)
- Windows static runtime builds green:
  - MinGW: `libhako_kernel.a` builds; sample link → `Result: 0`
  - MSVC: `hako_kernel.lib` builds; sample link → `Result: 0`
- Guide updated with “Static runtime (Windows) example” copy/paste
- Doctor: extended mode shows clear advice on missing allowlist/lib paths

Risks & Mitigations
- Legacy refs in `hako_kernel` may block Windows builds
  - Mitigate by gating under features, or providing plugin‑based alternatives
  - Keep dev stubs path as an interim; ensure examples still run

Next
- Harden `hako_kernel` Windows build (gate legacy refs; converge feature presets)
- Record success logs (MinGW/MSVC) and embed in the guide

