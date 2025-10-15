# Core Crates (hako_core_*) — Shared Semantics

Status: Introduced (Phase 15.7)

Purpose
- Provide a single, canonical implementation of core collection/string semantics
  that is reused by both the VM builtin path and plugins.
- Reduce duplication (VM vs plugin), keep behavior consistent, and pave the way
  for LLVM inlining or runtime externs.

Current crates
- `hako_core_string`
  - byte semantics by default:
    - `length_bytes(&str) -> i64`
    - `is_empty(&str) -> bool`
    - `index_of(&str, &str, from:i64) -> i64`
    - `last_index_of(&str, &str, from:i64) -> i64`
    - `substring_bytes(&str, start:i64, end:i64) -> String`
    - `char_at_byte(&str, idx:i64) -> String`
  - Notes: UTF‑8 code point (char) semantics may be added later under a guard.
    Current repo tests and plugins expect byte semantics.

- `hako_core_array`
  - Minimal helpers:
    - `length(len:usize) -> i64`
    - `slice_bounds(len:usize, start:i64, end:i64) -> (usize,usize)`

- `hako_core_map`
  - Minimal helpers:
    - `size(len:usize) -> i64`

Byte vs Code Point Policy
- Default is byte semantics. Existing VM behavior and current plugins use byte‑based
  substring/index operations. This ensures Selfhost and plugin‑on/off parity.
- Future option: Introduce char‑based variants guarded by env/feature flags and wire
  them via TypeRegistry so behavior is explicit and testable.

Adoption Plan
- String: VM and nyash‑string‑plugin already call `hako_core_string` for common ops.
- Array/Map: Start with size/bounds helpers; expand as slice/index logic is unified.
- LLVM: lower to externs or inline the `hako_core_*` functions when profitable.

Notes
- Keep the crates `no_std`‑friendly if needed later (no external deps by default).
- Document semantics and boundary rules; make changes explicit via tests before rollout.
