# Flow Blocks and Arrow Piping (Design Draft)

Status: design-only during the feature‑pause (no implementation)

Goal
- Make control/data flow visually obvious while keeping the core minimal.
- Core = anonymous `{ ... }` blocks + `->` chaining with `_` or `|args|` as the input placeholder.
- Always desugar to plain sequential let/if/call; zero new runtime constructs.

Core Syntax
- Serial (value flow):
  ```nyash
  { readConfig() }
    -> { |cfg| validate(cfg) }
    -> { |cfg| normalize(cfg) }
    -> { |cfg| save(cfg) }
  ```
- Placeholder short form:
  ```nyash
  { fetch() } -> { process(_) } -> { output(_) }
  ```
- If/Else with horizontal flow:
  ```nyash
  if cond -> { doA() } else -> { doB() }
  ```

Semantics
- `{ ... }` is an anonymous scope usable as expression or statement.
- `->` passes the left result as the first parameter of the right block.
- Left returns `Void` → right cannot use `_`/`|x|` (compile-time error in MVP spec).
- `_` and `|x,...|` are exclusive; mixing is an error.

Lowering (always zero-cost sugar)
- Chain desugars to temporaries and calls:
  ```nyash
  # {A} -> { |x| B(x) } -> { |y| C(y) }
  t0 = A();
  t1 = B(t0);
  t2 = C(t1);
  ```
- If/Else chain desugars to standard if/else blocks; merges follow normal PHI wiring rules.

Match normalization via guard chains
- Prefer a single readable form:
  ```nyash
  guard cond1 -> { A }
  guard cond2 -> { B }
  else        -> { C }
  ```
- Lowers to first-match if/else chain. No new pattern engine is introduced.

Range and CharClass guards (design)
- Range: `guard ch in '0'..'9' -> { ... }` → `('0' <= ch && ch <= '9')`.
- CharClass: `guard ch in Digit -> { ... }` → expands to ranges (e.g., '0'..'9').
- Multiple ranges combine with OR.

Formatting (nyfmt guidance)
- Align arrows vertically; one step per line:
  ```
  { fetch() }
    -> { validate(_) }
    -> { save(_) }
  ```
- Suggest factoring when chains exceed N steps; prefer naming a scope helper.

Observability (design only)
- `NYASH_FLOW_TRACE=1` prints the desugared steps (`t0=...; t1=...;`).

Constraints (MVP)
- No new closures; anonymous blocks inline when capture-free.
- Recursion not required; focus on linear/branching chains.
- ASI: treat `->` as a low-precedence line-continue operator.

Tests (syntax-only smokes; design)
- flow_linear: `read→validate→save` matches expected value.
- flow_placeholder: `{f()} -> { process(_) } -> { out(_) }`.
- flow_if: `if cond -> {A} else -> {B}` behaves like standard if.

Pause note
- Documentation and design intent only. Implementation is deferred until after the feature‑pause (post‑bootstrap).
