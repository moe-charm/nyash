# Ny JSON IR v0 — Minimal Spec (Stage‑2)

Status: experimental but stable for Phase‑15 Stage‑2. Input to `--ny-parser-pipe`.

Version and root
- `version`: 0
- `kind`: "Program"
- `body`: array of statements

Statements (`StmtV0`)
- `Return { expr }`
- `Extern { iface, method, args[] }` (optional; passes through to `ExternCall`)
- `Expr { expr }` (expression statement; side effects only)
- `Local { name, expr }` (Stage‑2)
- `If { cond, then: Stmt[], else?: Stmt[] }` (Stage‑2)
- `Loop { cond, body: Stmt[] }` (Stage‑2; while(cond) body)

Expressions (`ExprV0`)
- `Int { value }` where `value` is JSON number or digit string
- `Str { value: string }`
- `Bool { value: bool }`
- `Binary { op: "+"|"-"|"*"|"/", lhs, rhs }`
- `Compare { op: "=="|"!="|"<"|"<="|">"|">=", lhs, rhs }`
- `Logical { op: "&&"|"||", lhs, rhs }` (short‑circuit)
- `Call { name: string, args[] }` (function by name)
- `Method { recv: Expr, method: string, args[] }` (box method)
- `New { class: string, args[] }` (construct Box)
- `Var { name: string }`

CFG conventions (lowered by the bridge)
- If: create `then_bb`, `else_bb`, `merge_bb`. Both branches jump to merge if unterminated.
- Loop: `preheader -> cond_bb -> (body_bb or exit_bb)`, body jumps back to cond.
- Short‑circuit Logical: create `rhs_bb`, `fall_bb`, `merge_bb` with constants on fall path.
- All blocks end with a terminator (branch/jump/return).

PHI merging (current behavior)
- If: locals updated in `then`/`else` merge at `merge_bb` via `phi`.
  - Else欠落時は else 側に分岐前(base)を採用。
  - 片側にしか存在しない新規変数はスコープ外として外へ未伝播。
- Loop: `cond_bb` にヘッダ PHI を先置き（preheader/base と latch/body end を合流）。
- 目的: Stage‑2 を早期に安定化させるための橋渡し。将来（LoopForm= MIR18）では LoopForm からの逆Loweringで PHI を自動化予定。

Type meta (emitter/LLVM harness cooperation)
- `+` with any string operand → string concat path（handle固定）。
- `==/!=` with both strings → string compare path。

Special notes
- `Var("me")`: Bridge 既定では未定義エラー。デバッグ用に `NYASH_BRIDGE_ME_DUMMY=1` でダミー `NewBox{class}` を注入可（`NYASH_BRIDGE_ME_CLASS` 省略時は `Main`）。
- `--ny-parser-pipe` は stdin の JSON v0 を受け取り、MIR→MIR‑Interp 経由で実行する。

CLI/Env cheatsheet
- Pipe: `echo '{...}' | target/release/nyash --ny-parser-pipe`
- File: `target/release/nyash --json-file sample.json`
- Verbose MIR dump: `NYASH_CLI_VERBOSE=1`
- me dummy: `NYASH_BRIDGE_ME_DUMMY=1 NYASH_BRIDGE_ME_CLASS=ConsoleBox`

Examples

Arithmetic
```json
{"version":0,"kind":"Program","body":[
  {"type":"Return","expr":{
    "type":"Binary","op":"+",
    "lhs":{"type":"Int","value":1},
    "rhs":{"type":"Binary","op":"*","lhs":{"type":"Int","value":2},"rhs":{"type":"Int","value":3}}
  }}
]}
```

If with local + PHI merge
```json
{"version":0,"kind":"Program","body":[
  {"type":"Local","name":"x","expr":{"type":"Int","value":1}},
  {"type":"If","cond":{"type":"Compare","op":"<","lhs":{"type":"Int","value":1},"rhs":{"type":"Int","value":2}},
    "then":[{"type":"Local","name":"x","expr":{"type":"Int","value":10}}],
    "else":[{"type":"Local","name":"x","expr":{"type":"Int","value":20}}]
  },
  {"type":"Return","expr":{"type":"Var","name":"x"}}
]}
```
