# Phase 9.79b.1: Unified Registry IDs + Builder Slotting

Status: Planned
Owner: core-runtime
Target: Before Phase 10 (Cranelift JIT)
Last Updated: 2025-08-26

## Goals
- Introduce `BoxTypeId`/`MethodId` and stable method slot reservation in the unified registry.
- Resolve method names to slot IDs at MIR builder time when possible.
- Keep MIR instruction set stable (26) while enabling ID-based BoxCall.

## Scope
- Registry
  - Add numeric `BoxTypeId` mapping (type-name → id) and `(type_id, method)` → `slot` table.
  - Reserve low slots for universal methods: `0=toString`, `1=type`, `2=equals`, `3=clone`.
  - Provide `reserve_method_slot()`, `resolve_slot()` APIs.
- MIR Builder
  - When receiver type can be inferred, emit `BoxCall { method_id }` (slot ID) instead of name.
  - Add late-bind fallback path for unresolved sites (keeps current behavior).
  - Reserve user-defined instance methods slots deterministically (start at 4; universal [0..3]).
- Debug scaffolding
  - Add `MIRDebugInfo` container types (empty by default) for ID→name mapping (off by default).
- Docs
  - Update MIR design note to mention ID-based BoxCall with late-bind fallback.

## Deliverables
- New IDs and slot APIs in registry
- Builder emits `method_id` when resolvable
- Unit tests for slot reservation and universal slot invariants

## Non-Goals
- VM vtable/thunk dispatch (handled in 9.79b.2)
- PIC/JIT codegen

## Risks & Mitigations
- Slot consistency with inheritance: document rule “override keeps parent slot”; add test.
- Partial resolvability: ensure late-bind remains correct and does not regress semantics.

## Timeline
- 1–2 days

## Acceptance Criteria
- Tests pass; builder prints BoxCall with numeric `method_id` for resolvable sites.
- Universal methods occupy reserved slots across all types.
- No change to MIR opcode count (26) and existing dumps remain valid except for `method_id` where applicable.

## Roll-forward
- Proceed to 9.79b.2 (VM vtable/thunk + mono-PIC).
