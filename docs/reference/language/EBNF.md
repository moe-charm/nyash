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
