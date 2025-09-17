# Object Literal Sugar for Nyash (Draft)

Status: Proposal (target: Phase 9.79b or 9.80)
Author: core-runtime
Last Updated: 2025-08-26

## Goals
- Provide ergonomic inline construction of structured data for messaging and config.
- Keep parser stable and avoid ambiguity with blocks and control structures.
- Lower to existing Box types (MapBox/JSONBox) without breaking NyashBox trait.

## Syntax Candidates
- A) JSON-like literals
  - Example: `{ a: 1, b: "x", ok: true }`
  - Variant: Allow string keys without quotes; values are Nyash expressions.
- B) Explicit constructors with sugar keyword
  - Example: `map { a: 1, b: "x" }` → lowers to `new MapBox()` + sets
- C) JSON string auto-parse contextually
  - Example: `"{\"a\":1}"` auto-parsed where JSON is expected (too implicit → avoid as default)

Recommendation: Start with A) JSON-like object literal lowering to MapBox.

## Grammar Design (A)
- Token additions: `{`, `}`, `:`, `,` are already present.
- Production:
  - PrimaryExpr := ObjectLiteral | existing-primary
  - ObjectLiteral := `{` ObjMembers? `}`
  - ObjMembers := ObjPair (`,` ObjPair)* (`,`)?
  - ObjPair := Identifier `:` Expr
- Key constraints: Identifier (no quotes) initially; string-literal keys as follow-up.
- Values: Any expression; evaluation order left-to-right.

## Lowering Strategy
- `{ k1: v1, k2: v2 }` →
  - `tmp = new MapBox(); tmp.set("k1", (v1)); tmp.set("k2", (v2)); tmp`
- In AST builder: desugar ObjectLiteral into a sequence and final tmp reference.
- Side-effects: preserve evaluation order; each `vi` evaluated once.

## Ambiguity & Conflict Checks
- With blocks: Nyash does not use `{}` for statement blocks (loop uses `loop(cond) { }` but parser differentiates by context). Ensure lookahead disambiguates after `=` or in expression position.
- With `from`: no conflict.
- With `while/if` (obsolete): not applicable.

## Types & Interop
- MapBox chosen for mutability and existing rich API.
- Allow JSONBox interop later via `json{ ... }` form or helper.
- IntentBox constructor already accepts MapBox/JSONBox (implemented 2025-08-26).

## Examples
- `msg = new IntentBox("chat", { text: "hi", user: me.name })`
- `cfg = { retries: 3, verbose: true }`
- `headers = { Accept: "application/json", Host: host }`

## Validation & Tests
- Parser tests: empty `{}`, single/multi pairs, trailing comma, nested values, errors.
- Runtime tests: MapBox size/keys/values; evaluation order with side effects.
- Negative tests: `{ :1 }`, `{a 1}`, `{a:}`, `{a: 1, , b:2}`.

## Phased Rollout
- Phase 1 (behind feature flag `object_literal`): parser + lowering to MapBox; docs + examples.
- Phase 2: string-literal keys; nested object/array literals sugar (`[ ... ]`?) once array-literal is designed.
- Phase 3: JSONBox literal form `json{ ... }` (optional).

## Migration & Back-compat
- No breaking changes; literals are additive syntax.
- Existing code using JSON strings/MapBox continues to work.

## Open Questions
- Should we permit numeric/bool literals as keys? → No (keys stringify already).
- Trailing comma allowed? → Yes (developer-friendly).
- Pretty-printer/formatter impact? → Add simple rule to keep one-line small maps.

## Out of Scope (for this proposal)
- Array literal `[ ... ]` (future parallel RFC)
- Spread syntax, computed keys, deep merge helpers

## Appendix
- Related work: JS object literals, Rust maplit!, Lua tables, TOML inline tables.
- Risks: Parser precedence bugs → mitigate with unit tests + feature flag.

