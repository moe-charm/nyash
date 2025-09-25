# Nyash Core Principles — Minimal Syntax, Zero Runtime, Visual Flow

Status: design-only during the feature‑pause (no implementation changes)

Core (one page summary)
- Minimal syntax: `{ … }` + `->` + `|args|` or `_` for flow; guard chains as the canonical first-match form. No new `match` construct; normalize instead.
- Zero-runtime lowering: always desugar to `let/if/call/ret/phi`. No new opcodes, no implicit closures.
- Capture policy: capture by reference by default; mutation requires `mut` on the captured binding. Captures remain within the defining scope.
- ASI/format: treat `->` as a low-precedence line-continue. Formatter aligns arrows vertically.

Before/After (normalize view)
- Each example documents Ny → normalized Ny → MIR intent (design-only):
  1) Flow serial: `{read} -> { |x| validate(x) } -> { |x| save(x) }`
  2) Guard chain: `guard c1 -> {A}; guard c2 -> {B}; else -> {C}`
  3) If horizontal: `if cond -> {A} else -> {B}`
  4) Range pattern: `guard ch in '0'..'9' -> { ... }`
  5) Digit helper: `acc = acc*10 + ch.toDigitOrNull()` (null-guarded)

nyfmt alignment (visual flow)
```
{ fetch() }
  -> { validate(_) }
  -> { save(_) }
```

Domain demos (≤20 lines each)
- ETL pipeline: `read -> validate -> normalize -> save`
- Text/number parse: `guard ch in '0'..'9' -> { acc = acc*10 + ch.toDigitOrNull() }`
- Mini state machine: guard-first horizontal description with `else` fallback

Observability (spec hooks; design-only)
- `NYASH_SCOPE_TRACE=1|json`: enter/exit + captures (JSONL: `sid`, `caps`, `ret`).
- `NYASH_FLOW_TRACE=1`: desugared steps like `t0=B0(); t1=B1(t0);`.

Runtime/API additions (docs-only during the feature‑pause)
- `StringBox/Utf8Cursor`: `toDigitOrNull(base=10)`, `toIntOrNull()` — compile to simple comparisons/arithmetic.
- Guard sugar: Range (`'0'..'9'`) and CharClass (`Digit`, `AZ`, `az`, `Alnum`, `Space`) — compile to bound checks.

Acceptance & guardrails (feature‑pause)
- “No new grammar beyond sugar” and “no new VM opcodes” as hard rules during the feature‑pause.
- Golden texts (Ny → MIR fragments) to lock compatibility where practical.
- Lint proposals are documentation-only: single-use scope, long `->` chains, duplicated side effects.

- Related docs
- development/proposals/scope-reuse.md — local scope reuse blocks (MVP)
- development/design/legacy/flow-blocks.md — arrow flow + anonymous blocks
- reference/language/match-guards.md — guard chains + range/charclass sugar
- reference/language/strings.md — UTF‑8 first; proposed digit helpers
