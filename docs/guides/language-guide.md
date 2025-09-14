# Nyash Language Guide

Start here to learn Nyash language basics and find deeper references.

- Syntax Cheat Sheet: quick-reference/syntax-cheatsheet.md
- Full Language Reference (2025): reference/language/LANGUAGE_REFERENCE_2025.md
- Phase 12.7 Grammar (peek / ternary / sugar):
  - Overview: development/roadmap/phases/phase-12.7/grammar-specs/README.md
  - Tokens & Grammar: development/roadmap/phases/phase-12.7/ancp-specs/ANCP-Token-Specification-v1.md
- Sugar transformations (?., ??, |> ...): tools/nyfmt/NYFMT_POC_ROADMAP.md

Common Constructs
- Ternary operator: `cond ? then : else` (Phase 12.7); lowered to If-expression
- Peek expression: `peek value { lit => expr, else => expr }`
- Null-coalesce: `x ?? y` → `peek x { null => y, else => x }`
- Safe access: `a?.b` → `peek a { null => null, else => a.b }`

When you need the implementation details
- Tokenizer: src/tokenizer.rs
- Parser: src/parser/expressions.rs, src/parser/statements.rs
- Lowering to MIR: src/mir/builder/**
