# PoC Plan: TypeOp / WeakRef / Barrier Unification

Status: Draft (PoC design)
Last Updated: 2025-08-23

## Goals
- Reduce instruction surface without losing expressiveness or performance.
- Provide a feature-gated PoC to validate that consolidation is safe and measurable.

## Scope
- Unify TypeCheck + Cast → TypeOp (single instruction)
- Unify WeakNew + WeakLoad → WeakRef (single instruction)
- Unify BarrierRead + BarrierWrite → Barrier (single instruction)

## Out of Scope (PoC)
- Remap language syntax or external APIs
- Remove legacy instructions permanently (kept behind feature flags)

## Feature Flags (Cargo)
- `mir_typeop_poc`: enable TypeOp and map TypeCheck/Cast to it during build
- `mir_refbarrier_unify_poc`: enable WeakRef/Barrier unified ops and map legacy calls

## Mapping (Current → PoC)
- TypeCheck { value, expected_type } → TypeOp { op: Check, value, type } (bool)
- Cast { value, target_type } → TypeOp { op: Cast, value, type } (value)
- WeakNew { dst, box_val } → WeakRef { op: New, dst, box_val }
- WeakLoad { dst, weak_ref } → WeakRef { op: Load, dst, weak_ref }
- BarrierRead { ptr } → Barrier { op: Read, ptr }
- BarrierWrite { ptr } → Barrier { op: Write, ptr }

## Implementation Steps
1) MIR instruction additions
   - Add TypeOp/WeakRef/Barrier enums with minimal payloads
   - Keep legacy instructions compiled-in (no behavior change yet)
2) Builder mapping (feature-gated)
   - Under flags, emit unified instructions instead of legacy
3) VM execution mapping
   - Implement execute paths for TypeOp/WeakRef/Barrier
   - Legacy paths continue to work for fallback
4) Printer/Stats
   - Name new ops distinctly; ensure stats collection reflects consolidated ops
5) Tests
   - Snapshot tests for builder mapping (with/without flags)
   - VM exec parity tests for legacy vs unified

## Rollout / Migration
- Phase A (PoC): flags off by default, CI job with flags on
- Phase B (Dual): flags on by default in dev; legacy paths still supported
- Phase C (Switch): remove legacy or keep as aliases (no-emit) depending on impact

## Impact Areas
- `src/mir/instruction.rs` (add new ops; Display/used_values/dst_value)
- `src/mir/builder.rs` (conditional emit)
- `src/backend/vm.rs` (execution paths + stats key)
- `src/mir/printer.rs` (print new ops)
- Tests: MIR/VM/E2E minimal parity checks

## Acceptance Criteria
- All current tests pass with flags off (default)
- With flags on:
  - Unit/snapshot tests pass
  - vm-stats shows expected consolidation (TypeOp/WeakRef/Barrier vs legacy)
  - No regressions in FileBox/Net E2E under plugins

## Metrics to Watch
- vm-stats: proportion of TypeOp/WeakRef/Barrier vs legacy in representative scenarios
- Build time impact: negligible
- Code size: small reduction after removal

## Risks / Mitigations
- Risk: Unified ops obscure dataflow for some analyses
  - Mitigation: Verifier hooks to introspect TypeOp semantics; keep legacy printer names during PoC
- Risk: Plugins or external tooling tied to legacy names
  - Mitigation: MIR remains internal; external ABI unaffected

## Next Steps
- Land scaffolding (no behavior change)
- Add builder mapping behind flags
- Add VM execution behind flags
- Gate CI job to run PoC flags on Linux

