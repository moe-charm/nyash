# Scope Reuse Blocks (MVP Proposal)

Status: design-only during freeze (no implementation)

Summary
- Give short, reusable logic a name within the current scope without promoting it to a top-level function.
- Keep the core small: block body + postfix header sugar; desugar to local function + normal calls.
- Zero runtime cost: lowers to let/if/call/ret only (no new instructions/closures).

Syntax (postfix header; Nyash style)
- Block form (multi-statement):
  ```nyash
  { /* BODY */ } scope name(arglist?) (-> Ret)?
  // call within the same scope
  name(args)
  ```
- Expression form (one-liner):
  ```nyash
  => EXPR  scope name(arglist?) (-> Ret)?
  ```

Semantics
- Visibility: `name` is local to the defining scope; not exported.
- Capture: by reference by default. Mutating captured vars requires explicit `mut` on those bindings.
- Recursion: disallowed in MVP (can be lifted later).
- Errors/exits: same as regular functions (return/cleanup/catch apply at the function boundary).

Lowering (desugaring)
- Transform into a local function plus a local binding for convenience calls.
  ```nyash
  // { BODY } scope check(a:Int)->Str
  // ↓ (conceptual)
  let __cap_me = me; let __cap_locals = { /* needed refs */ };
  method __scope_check__(a:Int)->Str {
    return BODY
  }
  let check = (x) => __scope_check__(x)
  ```
- Captures are passed via hidden arguments or an environment box; no new VM opcodes.

Examples
```nyash
{ if x % 2 == 0 { return "even" } return "odd" } scope parity(x:Int)->StringBox

for i in range(0,10) {
  print(parity(i))
}
```

Safety rules (MVP)
- Capture: read-only by default; writes allowed only when the captured binding is declared `mut`.
- Name uniqueness: `scope name` must be unique within the scope.
- No cross-scope escape: values may be returned but the function reference itself is not exported.

Observability & Tooling
- Add trace toggles (design only):
  - `NYASH_SCOPE_TRACE=1|json` to emit enter/exit and capture lists as JSONL.
  - Example: `{ "ev":"enter","sid":42,"caps":["me","cfg","mut total"] }`.
- Lints (design only):
  - Single-use scope → suggest inline.
  - Excess captures → suggest narrowing.

Interactions
- Works with guard/with/await sugars (it’s just a call).
- Compatible with ASI and postfix aesthetics; no new top-level keywords beyond `scope` suffix.

Tests (syntax-only smokes; design)
- scope_basic: called twice → same result.
- scope_capture_read: reads `me/foo`.
- scope_capture_mut: mutation only allowed when `mut` is present.
- scope_with_catch_cleanup: postfix catch/cleanup applied at local-function boundary.

Freeze note
- This is documentation and design intent only. Implementation is deferred until after the freeze.

