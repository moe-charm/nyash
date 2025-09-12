# Resolver API (Minimal i64 Prototype)

Goals
- Centralize "ValueId → current-block value" resolution.
- Guarantee dominance by localizing values at the start of the block (before non-PHI).
- De-duplicate per (block, value) to avoid redundant PHIs/casts.

Design
- `Resolver` keeps small per-function caches keyed by `(BasicBlockId, ValueId)`.
- `resolve_i64(...)` returns an `i64`-typed `IntValue`, inserting PHI and casts as needed using sealed snapshots.
- Internally uses `flow::localize_to_i64(...)` for now; later, fold logic directly and add `resolve_ptr/resolve_f64`.

Usage (planned wiring)
- Create `let mut resolver = instructions::Resolver::new();` at function lowering start.
- Replace all integer value fetches in lowerers with `resolver.resolve_i64(...)`.
- Keep builder insertion discipline via `BuilderCursor`.

Next
- Add `resolve_ptr(...)` and `resolve_f64(...)` with same caching discipline.
- Migrate existing `localize_to_i64` call sites to the resolver.
- Enforce vmap direct access ban in lowerers (Resolver-only for reads).

Acceptance tie-in
- Combined with LoopForm: dispatch-only PHI + resolver-based value access → dominance violations drop to zero (A2.5).

