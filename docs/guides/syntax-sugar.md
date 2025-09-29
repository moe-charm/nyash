Syntax Sugar Guide (Phase 12.7+)

Overview
- Default: ON. Toggle with env `NYASH_SYNTAX_SUGAR_LEVEL={off|basic|full}` (default: full).
- Scope: parser-level desugaring only. Core semantics unchanged.

Levels
- off: no sugar.
- basic: pipeline `|>`, coalesce `??`, nullsafe `?.`, range `..`, array/map literals, trailing commas, numeric separators.
- full: basic + pipeline receiver `.m(...)`, placeholder `_` in `|>` RHS, raw strings.

Pipeline `|>`
- `x |> f(a,b)` desugars to `f(x,a,b)`.
- `x |> obj.m(a)` desugars to `obj.m(x,a)`.
- `x |> .m(a)` desugars to `x.m(a)` (receiver shorthand).
- Placeholder: `x |> f(_, k)` → replace single `_` with `x` (multiple `_` is a parse error).
- Precedence: function application and `.` bind tighter than `|>`.

Raw Strings
- `r"..."` (no escapes), `r#"..."#`, `r##"..."##` for nested quotes.
- Value is taken verbatim until the matching closing delimiter.

Trailing Commas
- Allowed in array/map literals and argument lists: `f(a,b,)`, `{k:1,}`, `[1,2,]`.

Numeric Separators
- Allow `_` inside decimal and float literals: `1_000_000`, `3.141_592`.

Env/CLI
- `NYASH_SYNTAX_SUGAR_LEVEL` controls sugar globally. `NYASH_FORCE_SUGAR=1` forces ON.
- Suggested dev flags: `NYASH_PRINT_DESUGARED=1` (future) to dump desugared code.

Notes
- Tap operator `|?>` and advanced variants (`?>`, `~>`, `||>`) are reserved for future and currently desugar like `|>`.

