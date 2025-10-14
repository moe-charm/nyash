# Nyash Grammar (Stage‑2 EBNF)

Status: Fixed for Phase 15 Stage‑2. Parser implementations (Rust/Python/Nyash selfhost) should conform to this subset.

program   := stmt* EOF

stmt      := 'return' expr
           | 'local' IDENT '=' expr
           | 'if' expr block ('else' block)?
           | 'loop' '('? expr ')' ? block
           | derive_decl                 ; gated by NYASH_MACRO_ENABLE=1 (planned/mvp)
           | for_macro                   ; gated by NYASH_MACRO_ENABLE=1 (planned/mvp)
           | flow_decl                 ; default ON (disable: NYASH_ENABLE_FLOW=0)
           | expr                         ; expression statement

block     := '{' stmt* '}'

expr      := logic
logic     := compare (('&&' | '||') compare)*
compare   := sum (( '==' | '!=' | '<' | '>' | '<=' | '>=' ) sum)?
sum       := term (('+' | '-') term)*
term      := unary (('*' | '/') unary)*
unary     := ('-' | '!' | 'not') unary | factor

factor    := INT
           | STRING
           | IDENT call_tail*
           | '(' expr ')'
           | 'new' IDENT '(' args? ')'
           | '[' args? ']'           ; Array literal (Stage‑1 sugar, gated)
           | '{' map_entries? '}'    ; Map literal (Stage‑2 sugar, gated)
           | match_expr              ; Pattern matching (replaces legacy peek)

match_expr := 'match' expr '{' match_arm+ default_arm? '}'
match_arm  := pattern guard? '=>' (expr | block) ','?
default_arm:= '_' '=>' (expr | block) ','?

pattern   := '_'
           | STRING | INT | 'true' | 'false' | 'null'
           | IDENT '(' IDENT? ')'           ; Type pattern e.g., StringBox(s)
           | '[' (IDENT (',' '..' IDENT)? )? ']'
           | '{' ( (STRING|IDENT) ':' IDENT (',' '..')? )? '}'
           | pattern '|' pattern            ; OR pattern (same arm)

guard     := 'if' expr

map_entries := (STRING | IDENT) ':' expr (',' (STRING | IDENT) ':' expr)* [',']

call_tail := '.' IDENT '(' args? ')'   ; method
           | '(' args? ')'             ; function call

args      := expr (',' expr)*

; ---- Flow (stateless namespace) — default ON; disable with NYASH_ENABLE_FLOW=0 ----
flow_decl := 'flow' IDENT '{' flow_member* '}'
flow_member := method_decl
; Forbidden inside flow: field declarations, 'birth'/'fini' methods, 'me' usage in method bodies

; ---- Macro syntax (planned; gated by NYASH_MACRO_ENABLE=1) ----
; Attribute derive for the following box declaration:
;   @derive('Equals', 'ToString', ...)
;   box Name { ... }
derive_decl := '@' 'derive' '(' (STRING (',' STRING)*)? ')' box_decl

; Foreach sugar:
;   @for (x in expr) { block }
;   @for (k, IDENT 'in' expr) { block }  ; map pairs
;   @for (IDENT 'in' expr RANGE expr) { block } ; range (end exclusive)
for_macro := '@' 'for' '(' IDENT (',' IDENT)? 'in' expr (RANGE expr)? ')' block

; Repeat sugar (counted loop) and assert sugar
repeat_macro := '@' 'repeat' '(' expr ')' block
assert_macro := '@' 'assert' '(' expr (',' expr)? ')'

Notes
- ASI: Newline is the primary statement separator. Do not insert a semicolon between a closed block and a following 'else'.
- Semicolon (optional): When `NYASH_PARSER_ALLOW_SEMICOLON=1` is set, `;` is accepted as an additional statement separator (equivalent to newline). It is not allowed between `}` and a following `else`.
- Do‑while: not supported by design. Prefer a single‑entry, pre‑condition loop normalized via sugar (e.g., `repeat N {}` / `until cond {}`) to a `loop` with clear break conditions.
- Short-circuit: '&&' and '||' must not evaluate the RHS when not needed.
- Unary minus has higher precedence than '*' and '/'.
- IDENT names consist of [A-Za-z_][A-Za-z0-9_]*
- Array literal is enabled when syntax sugar is on (NYASH_SYNTAX_SUGAR_LEVEL=basic|full) or when NYASH_ENABLE_ARRAY_LITERAL=1 is set.
- Map literal is enabled when syntax sugar is on (NYASH_SYNTAX_SUGAR_LEVEL=basic|full) or when NYASH_ENABLE_MAP_LITERAL=1 is set.
- Identifier keys (`{name: v}`) are Stage‑3 and require either NYASH_SYNTAX_SUGAR_LEVEL=full or NYASH_ENABLE_MAP_IDENT_KEY=1.
- Pattern matching: `match` replaces legacy `peek`. MVP supports wildcard `_`, literals, simple type patterns, fixed/variadic array heads `[hd, ..tl]`, simple map key extract `{ "k": v, .. }`, OR patterns, and guards `if`.
 - Macro syntax (planned): `@derive` and `@for` are recognized when macros are enabled. MVP limits `@for` to array iteration. See macro-syntax.md for details.

## Box Members (Phase‑15, env gate: NYASH_ENABLE_UNIFIED_MEMBERS; default ON)

This section adds a minimal grammar for Box members (a unified member model) without changing JSON v0/MIR. Parsing is controlled by env `NYASH_ENABLE_UNIFIED_MEMBERS` (default ON; set `0/false/off` to disable).

```
box_decl       := 'box' IDENT '{' member* '}'

member         := stored
                | computed
                | once_decl
                | birth_once_decl
                | method_decl
                | block_as_role      ; nyash-mode (block-first) equivalent

stored         := IDENT ':' TYPE ( '=' expr )?
                  ; stored property (read/write). No handlers supported.

computed       := IDENT ':' TYPE ( '=>' expr | block ) handler_tail?
                  ; computed property (read‑only). Recomputes on each read.

once_decl      := 'once' IDENT ':' TYPE ( '=>' expr | block ) handler_tail?
                  ; lazy once. First read computes and caches; later reads return cached value.

birth_once_decl:= 'birth_once' IDENT ':' TYPE ( '=>' expr | block ) handler_tail?
                  ; eager once. Computed during construction (before user birth), in declaration order.

method_decl    := IDENT '(' params? ')' ( ':' TYPE )? block handler_tail?

; nyash-mode (block-first) variant — gated with NYASH_ENABLE_UNIFIED_MEMBERS=1
block_as_role  := block 'as' ( 'once' | 'birth_once' )? IDENT ':' TYPE

handler_tail   := ( catch_block )? ( cleanup_block )?
catch_block    := 'catch' ( '(' ( IDENT IDENT | IDENT )? ')' )? block
cleanup_block  := 'cleanup' block

; Stage‑3 (Phase 1 via normalization gate NYASH_CATCH_NEW=1)
; Postfix handlers for expressions and calls
postfix_catch      := primary_expr 'catch' ( '(' ( IDENT IDENT | IDENT )? ')' )? block
postfix_cleanup    := primary_expr 'cleanup' block
```

Semantics (summary)
- stored: O(1) slot read; write via assignment. Initializer (if present) evaluates at construction once.
- computed: read‑only; each read evaluates the block; assignment is an error unless a setter is explicitly defined.
- once: first read evaluates the block and caches the value; subsequent reads return the cached value. On exception without a `catch`, the property becomes poisoned and rethrows on later reads (no retries).
- birth_once: evaluated before the user `birth` body, in declaration order; exceptions without a `catch` abort construction; cycles between `birth_once` members are an error.
- handlers: `catch/cleanup` are permitted for computed/once/birth_once/method blocks (Stage‑3), not for stored.

Lowering (no JSON v0 change)
- stored → slot
- computed → synthesize `__get_name():T { try body; catch; finally }`; reads of `obj.name` become `obj.__get_name()`
- once → add `__name: Option<T>` and emit `__get_name()` with first‑read initialization; on uncaught exception mark poisoned and rethrow on subsequent reads
- birth_once → add `__name: T` and insert initialization just before user `birth` in declaration order; handlers apply to each initializer
- method → existing method forms; optional postfix handlers lower to try/catch/finally

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

- Member‑level postfix catch/cleanup（Phase 15.6, gated）
  - Applies to computed/once/birth_once in the unified member model: see “Box Members”.
  - Gate: `NYASH_PARSER_STAGE3=1` (shared). Stored members do not accept handlers.

These constructs remain experimental; behaviour may degrade to no‑op in some backends until runtime support lands, as tracked in CURRENT_TASK.md.
