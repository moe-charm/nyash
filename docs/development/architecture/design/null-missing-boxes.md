# NullBox / MissingBox — Design and Rollout (Observe → Adopt)

Status: proposal accepted for dev-only observation; defaults unchanged

## Goals

- Make “absence of value” explicit and observable without changing prod behavior.
- Separate “explicit null” from “missing/unset” to improve diagnostics, reduce ambiguity, and simplify downstream policies.
- Stage the rollout via Observe → Adopt: enable metrics in dev, verify zero-diff on outputs, then adopt selectively.

## Terms

- Null = explicit no-value. Represented by NullBox (stringify: "null").
- Missing = absent/unset (missing key, uninitialized). Represented by MissingBox (stringify: "(missing)" in dev only; not surfaced in prod).
- Bottom/void (unreachable/side-effect-only) is NOT materialized as a box. Using it as a value is a bug (trap).

## Behavior (policy sketch)

- Equality: Null == Null → true. Missing: comparison is error.
- Ordering (<, <=, >, >=): Null and Missing → error.
- Arithmetic: Null propagates (returns Null) or errors when `NYASH_NULL_STRICT=1`. Missing → error.
- Coalesce `??`: Null ?? x = x, Missing ?? x = x.
- Safe-call `?.`: Null?.m() = Null; Missing?.m() = Missing (or error by policy).

Note: language operators `??`/`?.` may be introduced later; initial staging can use functions or Operator Boxes.

## Implementation plan (minimal, reversible)

1) Types (done, dev scope only)
   - Add `NullBox` (already present) and `MissingBox` (new) as first-class boxes.
   - No change to default value mapping: VM maps NullBox to VMValue::Void for backward compatibility.

2) Env toggles
   - `NYASH_NULL_MISSING_BOX=1`: enable observation path (no default behavior changes).
   - `NYASH_NULL_STRICT=1`: strict policy (operators error on null) — effective only when the first flag is enabled.

3) VM integration (dev-only observation)
   - Classification helpers recognize BoxRef(NullBox/MissingBox) for traces.
   - Print: Void and BoxRef(VoidBox) → "null"; BoxRef(NullBox) prints via box `toString` ("null"); MissingBox prints "(missing)" in dev.
   - Compare/Add: no default behavior changes; strict behavior is gated behind envs and will be staged later.

4) JSON/Node integration (later)
   - Optional: when flag is on, `object_get/array_get` may return `MissingBox` for absent entries, while `null` remains `NullBox`.
   - UI/print normalization can translate Missing → null at boundaries based on policy.

## Acceptance

- Defaults unchanged (prod): no output or semantic differences.
- Dev-only path provides metrics and safe diagnostics; quick/integration smokes remain green.
- After stability, selective adopt in Compare/Add can be enabled (Compare already adopted by default).

