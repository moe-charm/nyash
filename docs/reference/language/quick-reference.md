# Nyash Quick Reference (MVP)

Purpose
- One‑page practical summary for writing and implementing Nyash.
- Keep grammar minimal; clarify rules that often cause confusion.

Keywords (reserved)
- control: `if`, `else`, `loop`, `match`, `case`, `break`, `continue`, `return`
- decl: `flow`, `static`, `box`, `local`, `using`, `as`
- lit: `true`, `false`, `null`, `void`

Expressions and Calls
- Function call: `f(a, b)`
- Method call: `obj.m(a, b)` — internally normalized to function form: `Class.m(me: obj, a, b)`
  - Default‑ON（P4）: Known 受信者かつ関数が一意に存在する場合に正規化（userbox 限定）。
  - それ以外（Unknown/core/user‑instance）は安全に BoxCall へフォールバック（挙動不変）。
  - 環境で無効化: `NYASH_REWRITE_KNOWN_DEFAULT=0`（開発時の切替用）。
  - バックエンド（VM/LLVM/Ny）は統一形状の呼び出しを受け取る。
 - VM 実行ポリシー: ユーザーBoxの Instance BoxCall は開発のみ許容（prod 既定 = 不許可）。
    - env: `NYASH_VM_USER_INSTANCE_BOXCALL={0|1}`（既定: dev/ci=true, prod=false）
    - 規範: ビルダーが関数化（Instance→Function）し、ランタイム Instance BoxCall に依存しない。
  - 内部規範（String‑like 所属）: `length/len/substring/indexOf/lastIndexOf` は `StringBox` に所属。
    - Unknown/Instance/ParserBox/DebugBox/FileBox でこれらが現れた場合は、ビルダーが `StringBox` に正規化して解決する。
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
 - Enforce (dev flag): `NYASH_ASI_STRICT=1`（既定は互換・OFF）

Truthiness (boolean context)
- `Bool` → itself
- `Integer` → `0` is false; non‑zero is true
- `String` → empty string is false; otherwise true
- `Array`/`Map`/`Box` → non‑null is true（コンテナ/オブジェクトは truthy。サイズやフィールドは見ない）
- `null`/`void` → false

Equality and Comparison
- `==` and `!=` compare primitive values (Integer/Bool/String). No implicit cross‑type coercion.
- Box/Instance comparisons should use explicit methods (`equals`), or be normalized by the builder.
- Compare operators `< <= > >=` are defined on integers (MVP).
 - Enforce Box== guidance (dev flag): `NYASH_BOX_EQ_GUIDE_ERROR=1`

String and Numeric `+`
- If either side is `String`, `+` is string concatenation.
- If both sides are numeric, `+` is addition.
- Other mixes are errors (dev: warn; prod: error) — keep it explicit（必要なら `str(x)` を使う）。
 - Enforce (dev flag): `NYASH_PLUS_MIX_ERROR=1`（既定は互換・OFF）

String Literals
- Normal: `"hello\nworld"` — interprets escapes: `\"`, `\\`, `\n`, `\t`, `\r`
- Raw: `r"C:\path\file.txt"` — no escape interpretation (raw bytes)
- Raw with quotes: `r#"He said "Hello""#` — use `#` delimiters (can nest: `r##"..."##`)
- JSON processing: Use scanner boxes for robust parsing (escape-aware):
  - `selfhost/vm/boxes/string_scan.hako` — `find_unescaped()`, `scan_string_end()`
  - `selfhost/vm/boxes/json_scan.hako` — `seek_obj_end()`, `find_key_dual()` (plain/escaped)

Blocks and Control
- `if (cond) { ... } [else { ... }]`
- `loop (cond) { ... }` — minimal loop form
- `match (expr) { case ... }` — MVP (literals and simple type patterns)

Sugar Syntax (Phase 12.7+)
- Lambda: `fn(x, y) { x + y }` or `fn(x) { x * 2 }` (single expr, implicit return)
  - Use in: `array.map(fn(x) { x * 2 })`, `sort(fn(a,b) { a - b })`
- Result propagation: `data = readFile(path)?` — early return on error
- Postfix handlers: `doWork() catch(e) { handle(e) } cleanup { always() }`
- Match expression: `match ch { "0" => 0, "1" => 1, _ => -1 }`
  - Prefer over if-chain for lookup/dispatch

See also: docs/cookbook/quick-tips.md (practical examples)
Advanced: docs/development/roadmap/language-evolution/ (full roadmap)

Using / SSOT
- Dev/CI: file‑based `using` allowed for convenience.
- Prod: `hako.toml` only (compat: `nyash.toml`). Duplicate imports or alias rebinding is an error.

Errors (format)
- Always: `Error at line X, column Y: <message>`
- For tokenizer errors, add the reason and show one nearby line if possible.

Dev/Prod toggles (indicative)
- `NYASH_DEV=1` — developer defaults (diagnostics, tracing; behavior unchanged)
- `NYASH_USING=1` — enable using resolver (`NYASH_USING_STRATEGY={resolver|prelude}` for merge mode)
- `NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=1` — allow `main` as top‑level entry

Notes
- Keep the language small. Prefer explicit conversions (`int(x)`, `str(x)`, `bool(x)`) in standard helpers over implicit coercions.
- Builder rewrites method calls to keep runtime dispatch simple and consistent across backends.

Flow (stateless namespace)
- `flow Name { ... }` defines a stateless container of methods.
- Allowed: methods, local variables inside methods.
- Forbidden: fields, `birth`/`fini`, `new Name()`, `me` inside methods.
- Lowering intent: `Name.method(a, b)` → global `Name.method/2` (no BoxCall).
- Use for entry modules (Main.main) and utility groups.
