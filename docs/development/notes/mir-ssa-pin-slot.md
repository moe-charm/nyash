# MIR SSA Dominance: Pin ephemeral values to slots (dev note)

Context
- VM fallback (MIR interpreter) expects that, upon entering a block, every used SSA ValueId is defined in a dominating block or in this block (via Phi/Copy).
- User variables are tracked via `variable_map` and merged with existing `merge_modified_vars` / `normalize_if_else_phi` helpers. However, “ephemeral” expression values (not bound to variables) can cross block boundaries and break dominance.

Problem
- JSON tokenizer/parser frequently reuses the same char/string fragment across multiple branches in a function (e.g., scanner current char in if/elseif chains).
- Such reused operands may be produced in a predecessor block but then consumed in a new block without a dominating definition (no Phi/Copy), leading to “use of undefined value ValueId(..)” in the VM.

Solution (Pin + existing merges)
1) Pin ephemeral values to pseudo local slots before they are reused across blocks.
   - Add `pin_to_slot(v, hint)` to MirBuilder: emits a Copy to create a local SSA value and inserts it into `variable_map` under a generated slot name. From then on, the value participates in the normal variable merge (PHI) logic at control-flow merges.
   - Use in short-circuit lowering: after computing LHS, call `pin_to_slot(lhs, "@sc_lhs")` and use the pinned value in subsequent branches.

2) If-form (branch) reuse
   - For conditions (and inner patterns) where the same operand is likely checked repeatedly across then/else blocks, pin that operand before branching, so downstream uses come from the slot and get merged naturally.

3) Optional safety net (not enabled by default)
   - For single-predecessor blocks, materialize slot values at block entry with Copy to ensure a local definition exists. This is a coarse fallback and may be noisy; keep as a guarded option if needed.

Verifier (future)
- Add a dev-only MIR verification: for each non-Phi instruction operand, assert its definition dominates the use (or it was materialized in-block). Emit usable diagnostics to catch regressions early.

Acceptance
- JSON quick VM smokes pass without undefined value errors.
- Short-circuit and branch lowering remain spec-equivalent.

Rollback
- Pinning is additive and local. Remove calls to `pin_to_slot` to roll back behavior at specific sites.

