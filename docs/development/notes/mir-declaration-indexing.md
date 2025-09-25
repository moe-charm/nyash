Title: MIR Builder — Declaration Indexing (Two‑Phase) Design Note

Summary
- Purpose: eliminate order‑sensitivity for forward references during AST→MIR lowering without changing language semantics.
- Approach: run a lightweight “Phase A: declaration indexing” over the AST to collect symbols, then perform normal lowering as “Phase B”.

Scope
- Internal compiler detail (builder). No changes to syntax/semantics/flags.
- Applies to: user‑defined boxes (instance types) and static methods lookup for bare calls.

Why
- Previous behavior processed AST in appearance order. Forward references (e.g., new JsonParser() before its box declaration) required ad‑hoc preindex_* helpers.
- Centralizing indexing avoids proliferation (preindex_user_boxes, preindex_static_methods, …) and keeps lowering simple.

Design
- Phase A (index_declarations):
  - user_defined_boxes: collect non‑static Box names to decide constructor behavior (skip birth() for user types).
  - static_method_index: map method name → [(BoxName, arity)] to resolve bare calls in using‑merged sources.
- Phase B (lowering): unchanged logic uses the above indices.

Invariants
- No behavior change. Only ordering robustness improves.
- Indexing walks AST once (O(N)). Data kept in MirBuilder fields already present.

Implementation Notes
- Function: MirBuilder::index_declarations(&ASTNode)
- Called once at lower_root() entry. Existing ad‑hoc preindex_* replaced by this single pass.
- Keep deltas small and localized to builder lifecycle.

Testing
- Existing smokes cover forward references (using/JSON). No new flags required.
- Acceptance: no stray unresolved calls due to order, no change in MIR output types for the same sources.

