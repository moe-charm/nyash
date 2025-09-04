# ANCP v1 – Reversible Mapping (P0 subset)

Status: Preview (12.7‑C P0). Scope is the sugar subset already implemented and gated in 12.7‑B.

Goals
- Provide a clear, reversible mapping between Nyash sugar and canonical forms.
- Make round‑trip (original → canonical → ANCP → canonical → original) predictable for the subset.

Gating
- Runtime sugar is gated by `NYASH_SYNTAX_SUGAR_LEVEL=basic|full`.
- ANCP tools/nyfmt remain PoC/docs only at this stage.

Subset Mappings
- Pipeline `|>`
  - Nyash: `lhs |> f(a,b)` → Canonical: `f(lhs, a, b)`
  - Nyash: `lhs |> obj.m(a)` → Canonical: `obj.m(lhs, a)`
  - Round‑trip invariant: No change of call order or arity.

- Safe Access `?.`
  - Nyash: `a?.b` → Canonical (peek): `peek a { null => null, else => a.b }`
  - Nyash: `a?.m(x)` → Canonical: `peek a { null => null, else => a.m(x) }`
  - Round‑trip invariant: No change of receivers/args; only the null guard appears.

- Default `??`
  - Nyash: `a ?? b` → Canonical (peek): `peek a { null => b, else => a }`
  - Round‑trip invariant: Both branches preserved as‑is.

- Range `..`
  - Nyash: `a .. b` → Canonical: `Range(a, b)`
  - Round‑trip invariant: Closed form preserved; no inclusive/exclusive change.

- Compound Assign `+=, -=, *=, /=`
  - Nyash: `x += y` → Canonical: `x = x + y`（`x` は変数/フィールド）
  - Round‑trip invariant: Operator identity preserved; left target identical.

Examples (Before / Canonical / Round‑Trip)
1) Pipeline + Default
```
Before:     data |> normalize() |> transform() ?? fallback
Canonical:  peek transform(normalize(data)) { null => fallback, else => transform(normalize(data)) }
RoundTrip:  data |> normalize() |> transform() ?? fallback
```

2) Safe Access Chain
```
Before:     user?.profile?.name
Canonical:  peek user { null => null, else => peek user.profile { null => null, else => user.profile.name } }
RoundTrip:  user?.profile?.name
```

3) Range + Compound Assign
```
Before:     i += 1; r = 1 .. 5
Canonical:  i = i + 1; r = Range(1, 5)
RoundTrip:  i += 1; r = 1 .. 5
```

Notes
- Precise precedence handling is left to the parser; mappings assume already parsed trees.
- Full ANCP compact tokens will be documented in a separate spec revision.

