# Phase 15.77 — Frozen Toolchain Polish & Windows Plan

Scope
- Finalize Frozen v1 pipeline polish (doctor extended, multi-object linking).
- Add Windows linking plan (MinGW/Clang and MSVC/clang-cl), with helper scripts.

What’s in this repo now
- Doctor (extended): `tools/aot/doctor_frozen_v1.sh` — builds main and extra libs (Parser/MirBuilder/VM), links and runs.
- Windows helpers: `tools/aot/windows/` — README, MinGW and MSVC link wrappers, stub C.
- Guides updated: `docs/guides/frozen-toolchain.md` — Windows notes and examples.

Next
- Stabilize `hako_kernel` feature presets for Windows (MinGW→MSVC parity).
- Add a minimal Windows smoke (optional): link and run a tiny EXE using stub + `.o` produced on WSL.
- Record success logs (Result: 0) in the guide once parity is achieved.
