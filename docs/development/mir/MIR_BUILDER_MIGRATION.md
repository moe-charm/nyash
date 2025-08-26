# MIR Builder Migration Plan (builder -> builder_modularized)

Goal: Gradually switch from `src/mir/builder.rs` to the modular split in
`src/mir/builder_modularized/` without breaking default builds.

## Phases

1) Compatibility layer (done)
   - Keep default export `MirBuilder` from `builder.rs`.
   - Gate modularized builder behind feature `mir_modular_builder`.
   - Add helpers (`emit_type_check/cast/weak_new/weak_load/barrier_*`) in `builder.rs`.

2) Field-name alignment (next)
   - `builder_modularized/core.rs` uses provisional field names for instructions.
   - Align to current `MirInstruction`:
     - TypeOp: `{ op, value, ty }` instead of `{ operation, operand, type_info }`.
     - WeakRef/Barrier: use `WeakRef { dst, op, value }` and `Barrier { op, ptr }` forms if present.
   - Import enums via `use crate::mir::{TypeOpKind, WeakRefOp, BarrierOp};`.

3) Swap export (opt-in → default)
   - With `--features mir_modular_builder`, ensure `cargo check` passes.
   - After parity tests (printer/optimizer/verifier), flip default export to modularized.

## Notes

- No behavior changes intended during migration — only structural split.
- Keep logs behind `NYASH_BUILDER_DEBUG=1` to avoid noise.

