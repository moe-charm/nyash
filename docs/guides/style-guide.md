# Nyash Style Guide (Phase 15)

Goals
- Keep Nyash sources readable and structured. Favor simple, predictable formatting compatible with reversible formatting (nyfmt PoC).

Formatting
- Indent with 2 spaces (no tabs).
- Braces: K&R style (opening brace on the same line).
- Max line length: 100 characters (soft limit).
- One statement per line. Use semicolons only when placing multiple statements on one physical line.
- Blank lines: separate top‑level `box` declarations with one blank line; no trailing blank lines at file end.

Statements and ASI
- Newline is the primary separator. See `reference/language/statements.md`.
- Do not insert semicolons before `else`.
- When breaking expressions across lines, break after an operator or keep the expression grouped.

using / include
- Place all `using` lines at the top of the file, before code.
- One `using` per line; no trailing semicolons.
- Sort `using` targets alphabetically; group namespaces before file paths.
- Prefer `as` aliases for readability. Aliases should be `PascalCase`.
- Keep `include` adjacent to `using` group, sorted and one per line.

Naming (conventions for Nyash code)
- Boxes (types): `PascalCase` (e.g., `ConsoleBox`, `PathBox`).
- Methods/functions: `lowerCamelCase` (e.g., `length`, `substring`, `lastIndexOf`).
- Local variables: concise `lowerCamelCase` (e.g., `i`, `sum`, `filePath`).
- Constants (if any): `UPPER_SNAKE_CASE`.

Structure
- Top‑to‑bottom: `using`/`include` → static/box declarations → helpers → `main`.
- Keep methods short and focused; prefer extracting helpers to maintain clarity.
- Prefer pure helpers where possible; isolate I/O in specific methods.

Box layout
- 統一メンバ採用時（`NYASH_ENABLE_UNIFIED_MEMBERS=1`）の推奨順序:
  1) stored（格納プロパティ: `name: Type [= expr]`）
  2) computed / once / birth_once（読み専: `name: Type {}` / `once name: Type {}` / `birth_once name: Type {}`）
  3) methods（`birth` を含む）
- 既存表記でも同様に「フィールド（格納）を先頭にまとめる」。
- 先頭群の後ろはメソッドのみを記述する。
- メンバ間の空行・コメントは許可。アノテーション（将来）もメンバ直前/行末で許可。
- NG: 最初のメソッド以降に stored を追加すること（リンタ警告／厳格モードでエラー）。

Lifecycle（birth/unborn）
- 既定は auto‑birth。`new TypeBox(args)` は直後に `birth(args)` が自動実行される。
- 詳細設定が必要な場合は `TypeBox.unborn().withXxx(...).birth()?` を使用。
- `birth()` は冪等であるべき（2回目は no‑op）。失敗は Result で返し、フォールバックしない。
- unborn 状態での set/get/call は Fail‑Fast（開発時は詳細メッセージ、製品時は簡潔）。

良い例
```nyash
box Employee {
  // データ構造（フィールド）
  name: StringBox
  age: IntegerBox
  department: StringBox

  // ここからメソッド
  birth(n, a, d) { me.name = n; me.age = a; me.department = d }
  promote() { }
}
```

悪い例（NG）
```nyash
box Bad {
  id: IntegerBox
  method1() { }
  name: StringBox  // ❌ フィールドはメソッドの後に置けない
}
```

ツール
- 警告: 既定は警告（`NYASH_CLI_VERBOSE=1` で詳細を表示）。
- 厳格化: `NYASH_FIELDS_TOP_STRICT=1` でエラーに昇格（Runnerでチェック）。

Examples
```nyash
using core.std as Std
using "apps/examples/string_p0.nyash" as Strings

static box Main {
  escJson(s) {  // lowerCamelCase for methods
    local out = ""
    local i = 0
    local n = s.length()
    loop(i < n) {
      local ch = s.substring(i, i+1)
      if ch == "\\" { out = out + "\\\\" }
      else if ch == "\"" { out = out + "\\\"" }
      else { out = out + ch }
      i = i + 1
    }
    return out
  }

  main(args) {
    local console = new ConsoleBox()
    console.println("ok")
    return 0
  }
}
```

CI/Tooling
- Optional formatter PoC: see `docs/tools/nyfmt/NYFMT_POC_ROADMAP.md`.
- Keep smoke scripts small and fast; place them under `tools/`.
