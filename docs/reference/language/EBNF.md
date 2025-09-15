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

call_tail := '.' IDENT '(' args? ')'   ; method
           | '(' args? ')'             ; function call

args      := expr (',' expr)*

Notes
- ASI: Newline is the primary statement separator. Do not insert a semicolon between a closed block and a following 'else'.
- Short-circuit: '&&' and '||' must not evaluate the RHS when not needed.
- Unary minus has higher precedence than '*' and '/'.
- IDENT names consist of [A-Za-z_][A-Za-z0-9_]*

