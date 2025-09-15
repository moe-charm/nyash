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

