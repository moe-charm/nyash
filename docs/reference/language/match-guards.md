# Match Guards — Syntax and Lowering (MVP + Design Notes)

Status: reference + design additions during feature‑pause (no implementation changes)

Scope
- Guarded branches as a readable form of first-match selection.
- Canonical lowering target: if/else chain + PHI merges.

Syntax (MVP)
- Guard chain (first-match wins):
  ```nyash
  guard <cond> -> { /* then */ }
  guard <cond> -> { /* then */ }
  else         -> { /* else */ }
  ```
- Conditions may combine comparisons, `is/as` type checks, and literals with `&&` / `||`.

Lowering
- Always lowers to a linear if/else chain with early exit on first true guard.
- Merge points use normal PHI formation invariants (see `reference/mir/phi_invariants.md`).

Design additions (frozen; docs only)
- Range Pattern (sugar):
  - `guard x in '0'..'9' -> { ... }`
  - Lowers to: `('0' <= x && x <= '9')`.
  - Multiple ranges: `in A..B || C..D` → OR of each bound check.
- CharClass (predefined sets):
  - `Digit ≡ '0'..'9'`, `AZ ≡ 'A'..'Z'`, `az ≡ 'a'..'z'`, `Alnum ≡ Digit || AZ || az`, `Space ≡ ' '\t\r\n` (MVP set; expandable later).
  - `guard ch in Digit -> { ... }` expands to range checks.

Errors & Rules (MVP)
- Default `_` branch does not accept guards.
- Type guard succeeds inside the then-branch; bindings (e.g., `StringBox(s)`) are introduced at branch head.
- Short-circuit semantics follow standard branch evaluation (right side is evaluated only if needed).

Observability (design)
- `NYASH_FLOW_TRACE=1` may trace how guard chains desugar into if/else.

Notes
- This page describes existing guard semantics and adds range/charclass as documentation-only sugar during the feature‑pause.
