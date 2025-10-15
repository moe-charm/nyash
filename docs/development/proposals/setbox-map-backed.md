# Proposal: SetBox as Map<Key, Unit>

## Context
- We want a Set abstraction without duplicating Eq/Hash/determinism semantics.
- MapBox already provides key normalization, Eq/Hash checks, and deterministic mode support.

## Design
- Represent Set as Map<Key, Unit> (Unit is unobservable and cost‑free for users).
- Public API (minimal): add/remove/has/size/isEmpty/clear/toArray.
- Semantics:
  - Mutating methods return NullBox (consistent with Map).
  - Missing elements: `has=false`; `remove` is idempotent (no error).
  - Determinism and key constraints are inherited from Map (Fail‑Fast for non‑hashable keys under deterministic mode).

## Integration points
- MIR/Builder: lower `Set.*` methods to `Extern("nyrt.set.*")`.
- VM externs: implement `nyrt.set.*` via Map operations.
  - HostHandle path: use slots for Map (`has/get/set/size/clear/keys`).
  - Legacy path: inner MapBox field when present.
- Strict policy: no builtin fallback when plugin provider exists (policy=force).

## Phasing (small, reversible)
1) Land docs (this proposal; boxes/collections; plugin integration notes).
2) Add VM extern handlers (nyrt.set.*) as thin delegates to Map.
3) Add Builder normalization for `Set.*` → Extern.
4) Add 2–3 smokes (add/has/size; remove idempotent; toArray order under deterministic mode).
5) Verify plugin‑only build. Keep changes small and revertible.

## Future work
- Optional `fromArray` factory.
- Performance tuning: switch internal storage without ABI changes.

