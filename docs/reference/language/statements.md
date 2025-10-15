# Statement Separation and Semicolons

Status: Adopted for Phase 15.3+; parser implementation is staged.

Policy
- Newline as primary statement separator.
- Semicolons are optional and only needed when multiple statements appear on one physical line.
- Minimal ASI (auto semicolon insertion) rules to avoid surprises. Semicolons are accepted by default.

Rules (minimal and predictable)
- Newline ends a statement when:
  - Parenthesis/brace/bracket depth is 0, and
  - The line does not end with a continuation token (`+ - * / . ,` etc.).
- Newline does NOT end a statement when:
  - Inside any open grouping `(...)`, `[...]`, `{...}`; or
  - The previous token is a continuation token.
- `return/break/continue` end the statement at newline unless the value is on the same line or grouped via parentheses.
- `if/else` (and similar paired constructs): do not insert a semicolon between a block and a following `else`.
- One‑line multi‑statements are allowed with semicolons: `x = 1; y = 2; print(y)`.
- Method chains can break across lines after a dot: `obj\n  .method()` (newline treated as whitespace).

Style guidance
- Prefer newline separation for readability.
- Use semicolons for multiple statements on a single line; both separators are valid.

Examples
```nyash
// Preferred (no semicolons)
local x = 5
x = x + 1
print(x)

// One line with multiple statements (use semicolons)
local x = 5; x = x + 1; print(x)

// Line continuation by operator
local n = 1 +
          2 +
          3

// Grouping across lines
return (
  1 + 2 + 3
)

// if / else on separate lines without inserting a semicolon
if cond {
  x = x - 1
}
else {
  print(x)
}

// Dot chain across lines
local v = obj
  .methodA()
  .methodB(42)
```

Implementation notes (parser)
- Tokenizer keeps track of grouping depth.
- At newline, attempt ASI only when depth==0 and previous token is not a continuation.
- To disable parsing of semicolons (dev/testing), set `NYASH_PARSER_ALLOW_SEMICOLON=0`.
- Error messages should suggest adding a continuation token or grouping when a newline unexpectedly ends a statement.

Parser dev notes (Stage‑1/2)
- return + newline: treat bare `return` as statement end. To return an expression on the next line, require grouping with parentheses.
- if/else: never insert a semicolon between a closed block and `else` (ASI禁止箇所)。
- Dot chains: treat `.` followed by newline as whitespace (line continuation)。
- One‑line multi‑statements: accept `;` as statement separator, but formatter should prefer newlines.
- Unary minus: disambiguate from binary minus; implement after Stage‑1（当面は括弧で回避）。
