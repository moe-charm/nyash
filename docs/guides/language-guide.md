# Nyash Language Guide

Start here to learn Nyash language basics and find deeper references.

- Syntax Cheat Sheet: quick-reference/syntax-cheatsheet.md
- Full Language Reference (2025): reference/language/LANGUAGE_REFERENCE_2025.md
- Phase 12.7 Grammar (match / ternary / sugar):
  - Overview: development/roadmap/phases/phase-12.7/grammar-specs/README.md
  - Tokens & Grammar: development/roadmap/phases/phase-12.7/ancp-specs/ANCP-Token-Specification-v1.md
- Sugar transformations (?., ??, |> ...): tools/nyfmt/NYFMT_POC_ROADMAP.md

Common Constructs
- Ternary operator: `cond ? then : else` (Phase 12.7); lowered to If-expression
- Match expression (pattern matching): `match value { pat => expr, _, ... }`
- Null-coalesce: `x ?? y` → `match x { null => y, _ => x }`
- Safe access: `a?.b` → `match a { null => null, _ => a.b }`

Minimal Examples
- Ternary
  ```nyash
  static box Main {
    main(args) {
      local a = 3
      local b = 5
      // Nested ternary is supported
      local v = (a < b) ? ((b < 0) ? 40 : 50) : 60
      return v
    }
  }
  ```
- Match as expression block (last expression is the value)
  ```nyash
  static box Main {
    main(args) {
      local d = "1"
      // Each arm can be a block; the last expression becomes the value
      local dv = match d {
        "0" => { print("found zero") 0 }
        "1" => { print("found one") 1 }
        _ => { print("other") 0 }
      }
      return dv
    }
  }
  ```

must_use Notes
- Match arms are expressions. When using a block arm `{ ... }`, the last expression is the resulting value; statements without a final expression yield no usable value.
- Ternary is an expression; ensure both branches are type-compatible at MIR level (e.g., both yield integer or both yield string handle in current phase).

When you need the implementation details
- Tokenizer: src/tokenizer.rs
- Parser: src/parser/expressions.rs, src/parser/statements.rs
- Lowering to MIR: src/mir/builder/**
Statement Separation (Semicolons)
- Newline separates statements by default; semicolons are optional.
- Use semicolons only when placing multiple statements on one line.
- Minimal ASI rules: newline does not end a statement when the line ends with an operator/dot/comma, or while inside grouping.
