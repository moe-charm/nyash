# Nyash Quick Reference (MVP)

Purpose
- One‑page practical summary for writing and implementing Nyash.
- Keep grammar minimal; clarify rules that often cause confusion.

Keywords (reserved)
- control: `if`, `else`, `loop`, `match`, `case`, `break`, `continue`, `return`
- decl: `static`, `box`, `local`, `using`, `as`
- lit: `true`, `false`, `null`, `void`

Expressions and Calls
- Function call: `f(a, b)`
- Method call: `obj.m(a, b)` — internally normalized to function form: `Class.m(me: obj, a, b)`
  - Default‑ON（P4）: Known 受信者かつ関数が一意に存在する場合に正規化（userbox 限定）。
  - それ以外（Unknown/core/user‑instance）は安全に BoxCall へフォールバック（挙動不変）。
  - 環境で無効化: `NYASH_REWRITE_KNOWN_DEFAULT=0`（開発時の切替用）。
  - バックエンド（VM/LLVM/Ny）は統一形状の呼び出しを受け取る。
- Member: `obj.field` or `obj.m`

Display & Conversion
- Human‑readable display: `str(x)`（推奨）/ `x.str()`
  - 既存の `toString()` は `str()` に正規化（Builder早期リライト）。
  - 互換: 既存の `stringify()` は当面エイリアス（内部で `str()` 相当へ誘導）。
- Debug表示（構造的・安定）: `repr(x)`（将来導入、devのみ）
- JSONシリアライズ: `toJson(x)`（文字列）/ `toJsonNode(x)`（構造）

Operators (precedence high→low)
- Unary: `! ~ -`
- Multiplicative: `* / %`
- Additive: `+ -`
- Compare: `== != < <= > >=`
- Logical: `&& ||` (short‑circuit, side‑effect aware)

Semicolons and ASI (Automatic Semicolon Insertion)
- Allowed to omit semicolon at:
  - End of line, before `}` or at EOF, when the statement is syntactically complete.
- Not allowed:
  - Line break immediately after a binary operator (e.g., `1 +\n2`)
  - Ambiguous continuations; parser must Fail‑Fast with a clear message.

Truthiness (boolean context)
- `Bool` → itself
- `Integer` → `0` is false; non‑zero is true
- `String` → empty string is false; otherwise true
- `Array`/`Map` → non‑null is true (size is not consulted)
- `null`/`void` → false

Equality and Comparison
- `==` and `!=` compare primitive values (Integer/Bool/String). No implicit cross‑type coercion.
- Box/Instance comparisons should use explicit methods (`equals`), or be normalized by the builder.
- Compare operators `< <= > >=` are defined on integers (MVP).

String and Numeric `+`
- If either side is `String`, `+` is string concatenation.
- If both sides are numeric, `+` is addition.
- Other mixes are errors (dev: warn; prod: error) — keep it explicit（必要なら `str(x)` を使う）。

Blocks and Control
- `if (cond) { ... } [else { ... }]`
- `loop (cond) { ... }` — minimal loop form
- `match (expr) { case ... }` — MVP (literals and simple type patterns)

Using / SSOT
- Dev/CI: file‑based `using` allowed for convenience.
- Prod: `nyash.toml` only. Duplicate imports or alias rebinding is an error.

Errors (format)
- Always: `Error at line X, column Y: <message>`
- For tokenizer errors, add the reason and show one nearby line if possible.

Dev/Prod toggles (indicative)
- `NYASH_DEV=1` — developer defaults (diagnostics, tracing; behavior unchanged)
- `NYASH_ENABLE_USING=1` — enable using resolver
- `NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1` — allow `main` as top‑level entry

Notes
- Keep the language small. Prefer explicit conversions (`int(x)`, `str(x)`, `bool(x)`) in standard helpers over implicit coercions.
- Builder rewrites method calls to keep runtime dispatch simple and consistent across backends.
