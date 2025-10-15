# Macro Syntax (Planned MVP)

Status: Planned for Phase 16 bring‑up; Rust parser support lands first as thin, deterministic sugar. Expansion remains semantics‑preserving and lowers into existing forms. Feature is gated by `NYASH_MACRO_ENABLE=1`.

Scope (MVP)
- `@derive('Equals','ToString','Debug')` placed immediately before a `box` declaration.
- `@for (x in expr) { ... }` array iteration sugar — lowers to a counted loop.
- `@for (k, v in map) { ... }` map iteration sugar — lowers via `keys()` + `get(k)`.
- `@for (i in start..end) { ... }` range sugar — lowers to a counted loop (end exclusive).
 - `@repeat(n) { ... }` counted loop sugar — lowers to a counted loop.
- `@assert(cond[, msg])` — lowers to `if (!cond) { throw msg }`. 

Gates and Profiles
- Gate: `NYASH_MACRO_ENABLE=1` (default ON in dev/ci profiles; see guides/macro-profiles.md)
- Strict behavior: failures in macro parsing/expansion are fail‑fast (diagnostic with line number).

## @derive

Grammar
```
@derive('Equals', 'ToString', ...)
box Name {
  public: field1: T, field2: U
  // user methods...
}
```

Expansion (deterministic)
- Injects missing methods onto the following `box` only. Existing methods are never overwritten.
- `Equals` →
  - `equals(other)` method that compares public fields structurally.
  - Zero‑field boxes get `equals(_) { return false }` (identity handled elsewhere).
- `ToString` →
  - `toString()` method yielding `Name(f1, f2, ...)` using public field order.

Notes
- Only public fields participate in derives (no private invariants leaked).
- Additional derives are reserved for later (e.g., `Clone`). Unknown names are an error in strict mode.

## @for (arrays / map pairs / ranges)

Grammar
```
@for (x in seq) {
  // body using x
}
```

Lowering (arrays)
```
local __ny_seq = <seq>;
local __ny_i = 0;
loop(__ny_i < __ny_seq.length()) {
  local x = __ny_seq.get(__ny_i);
  // body
  __ny_i = __ny_i + 1;
}
```

Maps (pairs)
```
local __ny_map = <map>;
local __ny_keys = __ny_map.keys();
local __ny_i = 0;
loop(__ny_i < __ny_keys.length()) {
  local k = __ny_keys.get(__ny_i);
  local v = __ny_map.get(k);
  // body
  __ny_i = __ny_i + 1;
}
```

Ranges (end exclusive)
```
local __ny_start = start;
local __ny_end = end;
local __ny_i = __ny_start;
loop(__ny_i < __ny_end) {
  local i = __ny_i;
  // body
  __ny_i = __ny_i + 1;
}
```

## @repeat / @assert

Grammar
```
@repeat(n) {
  // body
}
```

Lowering
```
local __ny_n = n;
local __ny_i = 0;
loop(__ny_i < __ny_n) {
  // body
  __ny_i = __ny_i + 1;
}
```

Diagnostics
- `@derive` without a following `box` → error.
- Strict mode: unknown derive names, invalid @for forms (e.g., wrong arity), or misplaced @derive/@assert will fail fast with diagnostics.

References
- EBNF: reference/language/EBNF.md (macro section)
- Profiles: guides/macro-profiles.md
- Capabilities (user macros): reference/macro/capabilities.md (syntactic macros do not require sandbox caps)
