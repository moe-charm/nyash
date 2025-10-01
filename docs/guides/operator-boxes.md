Operator Boxes (MVP) — Stringify

Overview
- Goal: Extend “Everything is Box” to implicit operations by modeling them as explicit, observable Boxes.
- Scope (MVP): Stringify operation used by print paths.
- Status: Dev‑only; opt‑in via env flag; backward compatible.

Quick Run
- JSON only (Roundtrip + Nested): `./tools/opbox-json.sh`
- Quick suite (dev, light preflight): `./tools/opbox-quick.sh`

Flags
- NYASH_OPERATOR_BOX_STRINGIFY=1  Enable StringifyOperator on print paths
- NYASH_OPERATOR_BOX_COMPARE=1    Enable CompareOperator (observer) on compare ops
- NYASH_OPERATOR_BOX_ADD=1        Enable AddOperator (observer) on Add binop
- NYASH_OPERATOR_BOX_ALL=1        Enable auto-prelude injection for all operator modules (runtime convenience)
- NYASH_USING_STRATEGY=prelude    Required to AST‑merge the operator modules automatically (compat: NYASH_USING_AST=1)
  
Parser tokens
- Tokenizer accepts `~`, `<<`, `>>` in non‑strict mode (default). In strict_12_7, shift tokens are gated.
- Unary `~x` parses to `UnaryOp(BitNot, x)`; binary `<< >> & | ^` parse via existing bit/shift rules.

Behavior
- Stringify: When the flag is ON, VM attempts to call `StringifyOperator.apply/1` before falling back to the legacy `to_string()`.
- Compare (observer): When the flag is ON, VM calls `CompareOperator.apply/3` with `(op, a, b)` for observability, then performs normal compare semantics (operator result is ignored for now).
- Add (observer): When the flag is ON, VM calls `AddOperator.apply/2` with `(a, b)` for observability, then performs normal add semantics (operator result is ignored for now).
- The operators live under `apps/lib/std/operators/` and are auto‑injected into the AST prelude (dev only) so functions are materialized without changing user sources.

Operator definitions (MVP)
- Stringify — `apps/lib/std/operators/stringify.nyash`
  - If `value` has `stringify()`, call it; else return `"" + value`.
- Compare — `apps/lib/std/operators/compare.nyash`
  - `apply(op, a, b)`; observer‑only for now, returns `true` and the VM performs the real compare.
- Add — `apps/lib/std/operators/add.nyash`
  - `apply(a, b)`; observer‑only for now, VM performs the real addition.
- Sub/Mul/Div/Mod — `apps/lib/std/operators/{sub,mul,div,mod}.nyash`
  - `apply(a, b)`; direct evaluation.
- Bitwise/Shifts — `apps/lib/std/operators/{bitand,bitor,bitxor,shl,shr}.nyash`
  - `apply(a, b)`; direct evaluation on integers.
- Unary — `apps/lib/std/operators/{neg,not,bitnot}.nyash`
  - `apply(a)`; negate / logical-not / bitwise-not.

Design Notes
- Backward compatible: existing programs run unchanged with the flag OFF (default).
- Observability: with `NYASH_BOX_TRACE=1`, calls to the operator are visible as JSON lines (stderr), aiding diagnostics.
- Rollback is trivial: remove the resolver injection and the two VM hooks; delete the operator file.

Future Work (Optional / Later)
- Elevate Compare/Add from observer to authoritative semantics behind stricter flags, once parity is verified.
- Unify lowering (sugar → operator boxes) after VM and Builder parity are green.
- Add per-operator adopt flags if needed for runtime switching; current builder call lowering already routes via operator boxes.

Builder lowering (centralized)
- Master flag: `NYASH_BUILDER_OPERATOR_BOX_ALL_CALL=1` lowers all arithmetic/compare/unary ops to `*Operator.apply` calls in one place (src/mir/builder/ops.rs).
- Reentrancy guard: when building inside `*Operator.apply`, lowering is skipped to avoid recursion; falls back to direct MIR op.
- Metadata: builder annotates result types (Integer/String/Bool) to preserve `value_types` so downstream analysis remains stable.
