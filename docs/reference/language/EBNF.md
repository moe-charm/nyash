# Nyash Grammar (Stage‑2 EBNF)

Status: Fixed for Phase 15 Stage‑2. Parser implementations (Rust/Python/Nyash selfhost) should conform to this subset.

program   := stmt* EOF

stmt      := 'return' expr
           | 'local' IDENT '=' expr
           | 'if' expr block ('else' block)?
           | 'loop' '('? expr ')' ? block
           | expr                         ; expression statement

block     := '{' stmt* '}'

expr      := logic
logic     := compare (('&&' | '||') compare)*
compare   := sum (( '==' | '!=' | '<' | '>' | '<=' | '>=' ) sum)?
sum       := term (('+' | '-') term)*
term      := unary (('*' | '/') unary)*
unary     := '-' unary | factor

factor    := INT
           | STRING
           | IDENT call_tail*
           | '(' expr ')'
           | 'new' IDENT '(' args? ')'
           | '[' args? ']'           ; Array literal (Stage‑1 sugar, gated)
           | '{' map_entries? '}'    ; Map literal (Stage‑2 sugar, gated)

map_entries := (STRING | IDENT) ':' expr (',' (STRING | IDENT) ':' expr)* [',']

call_tail := '.' IDENT '(' args? ')'   ; method
           | '(' args? ')'             ; function call

args      := expr (',' expr)*

Notes
- ASI: Newline is the primary statement separator. Do not insert a semicolon between a closed block and a following 'else'.
- Short-circuit: '&&' and '||' must not evaluate the RHS when not needed.
- Unary minus has higher precedence than '*' and '/'.
- IDENT names consist of [A-Za-z_][A-Za-z0-9_]*
- Array literal is enabled when syntax sugar is on (NYASH_SYNTAX_SUGAR_LEVEL=basic|full) or when NYASH_ENABLE_ARRAY_LITERAL=1 is set.
- Map literal is enabled when syntax sugar is on (NYASH_SYNTAX_SUGAR_LEVEL=basic|full) or when NYASH_ENABLE_MAP_LITERAL=1 is set.
- Identifier keys (`{name: v}`) are Stage‑3 and require either NYASH_SYNTAX_SUGAR_LEVEL=full or NYASH_ENABLE_MAP_IDENT_KEY=1.

## Stage‑3 (Gated) Additions

Enabled when `NYASH_PARSER_STAGE3=1` for the Rust parser (and via `--stage3`/`NYASH_NY_COMPILER_STAGE3=1` for the selfhost parser):

- try/catch/cleanup
  - `try_stmt := 'try' block ('catch' '(' (IDENT IDENT | IDENT | ε) ')' block) ('cleanup' block)?`
    - MVP policy: single `catch` per `try`。
    - `(Type var)` or `(var)` or `()` are accepted for the catch parameter。

- Block‑postfix catch/cleanup（Phase 15.5）
  - `block_catch := '{' stmt* '}' ('catch' '(' (IDENT IDENT | IDENT | ε) ')' block)? ('cleanup' block)?`
  - Applies to standalone block statements. Do not attach to `if/else/loop` structural blocks (wrap with a standalone block when needed).
  - Gate: `NYASH_BLOCK_CATCH=1` (or `NYASH_PARSER_STAGE3=1`).
- throw
  - `throw_stmt := 'throw' expr`

- Method‑level postfix catch/cleanup（Phase 15.6, gated）
  - `method_decl := 'method' IDENT '(' params? ')' block ('catch' '(' (IDENT IDENT | IDENT | ε) ')' block)? ('cleanup' block)?`
  - Gate: `NYASH_METHOD_CATCH=1`（または `NYASH_PARSER_STAGE3=1` と同梱）

These constructs remain experimental; behaviour may degrade to no‑op in some backends until runtime support lands, as tracked in CURRENT_TASK.md.
