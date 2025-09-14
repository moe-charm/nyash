# Parser MVP Stage-2 Design (Phase 15)

Scope
- Expand Stage-1 (const/binop/return) to cover the minimal everyday surface:
  - local bindings: `local x = expr`
  - if/else: `if cond { ... } else { ... }`
  - loop: `loop(cond) { ... }` and `loop cond { ... }`
  - call: `f(a, b)` (free function)
  - method: `obj.method(a, b)`
  - constructor: `new BoxType(args)`
  - implicit receiver `me` in box methods
  - String methods: `substring`, `length`, `lastIndexOf`

Grammar Sketch (incremental)
- Expression
  - precedence keeps Phase 12.7: ternary inside pipe; ternary lowers to if/branch
  - postfix: method call, field access
  - primary: literals, identifiers, parenthesized
- Statement
  - `local ident = expr` (no rebind in Stage-2)
  - expression statement (for side effects)
  - return

AST Nodes (delta)
- Add nodes for Local, If, Loop, Call, MethodCall, New, Identifier, Return.
- Keep Peek and Ternary in the AST; lower to MIR via the same builder hooks.

Lowering to MIR JSON v0
- Local → assign to fresh value id, store into vmap
- If/Ternary → compare + branch blocks; PHI for join values when used as expression
- Loop → cond block → body block → back-edge; loop-carried values explicitly phied
- Call → `call { func: "name", args: [...] }`
- Method → `boxcall { box: vid, method: "name", args: [...] }`
- New → `newbox { type: "BoxType", args: [...] }`
- String ops
  - `length()` → i64 length on string handle (VM: Python len; LLVM: handle→ptr at call sites only)
  - `substring(a,b)` → `nyash.string.substring_hh` equivalent
  - `lastIndexOf(x)` → returns i64 index or -1

Type Meta (emitted with JSON where needed)
- Compare: when both sides are string → `cmp_kind: "string"`
- BinOp `+`: if `dst_type = { kind: "handle", box_type: "StringBox" }`, force concat_hh
- Known APIs: annotate `dst_type` for Console.println (i64), dirname/join/read as `StringBox(handle)`

Policy & Invariants
- Resolver-only: do not wire PHIs in-lowering; snapshots + `finalize_phis` bind incomings
- Strings: inter-block are handle(i64) only; i8* materialized at call sites
- PHI placeholders: created at block heads from JSON-declared phi

Acceptance
- Parity on:
  - `apps/tests/min_str_cat_loop/main.nyash`
  - `apps/tests/esc_dirname_smoke.nyash`
  - Stage-2 new smokes (ternary nested, peek return)
- `tools/parity.sh --lhs pyvm --rhs llvmlite --show-diff <app>` → green

Notes
- Keep Stage-2 additive and test-driven; avoid broad refactors.
- Future: numeric methods, map/array minimal, and using/namespace gate.
