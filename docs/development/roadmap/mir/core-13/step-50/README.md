# MIR step‑50: Final Reference Sync after Core Flip

Status: Done

Purpose: After the Core‑15→Core‑13 flip is complete in code/tests, perform a last wave of documentation alignment across top‑level entry points and user‑facing docs.

## Scope
- Update top‑level docs to reflect Core‑13 as canonical minimal kernel:
  - `README.md` / `README.ja.md` (MIR summary snippet)
  - `docs/reference/mir/INSTRUCTION_SET.md` (fix counts/maps; remove migration disclaimers)
  - `docs/reference/architecture/*` (Core naming and diagrams)
- Add CHANGELOG note for the flip.
- DEV quickstart and contributor docs: link to Core‑13 reference and validation tests.

## Preconditions
- Tests enforce Core‑13 instruction count and legacy‑op forbiddance (see `src/mir/instruction_introspection.rs` and `src/tests/mir_core13_normalize.rs`).
- VM/JIT/AOT backends accept the reduced set (or have shims documented if not yet).

## Validation
- `cargo test` green with Core‑13 enforcement.
- `src/mir/instruction_introspection.rs` asserts exactly 13 in `core13_instruction_count_is_13`.
- `src/tests/mir_core13_normalize.rs` validates Array/Ref normalization to BoxCall.

## Rollback Plan
- Keep the Core‑15 reference/notes in `docs/development/roadmap/` (archive) for historical context.
