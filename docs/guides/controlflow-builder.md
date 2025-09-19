# ControlFlowBuilder — If/Match 正規化ビルダー（コンパイル時メタ）

目的
- If/Match の正規化（join 変数導入・scrutinee 一回評価・ガード合成・単一 PHI 群）を安全に一貫化するためのビルダー。
- 生成物は AST JSON v0 の文字列（マクロ内で使用）。実行時コストはゼロ。

設計原則
- 合流点は必ず join 変数（gensym）で収束（空 PHI を避け、PHI は合流ブロック先頭へ誘導）。
- 条件/scrutinee は 1 回だけ評価してローカルへ束ねる。
- 構造は If/Match → If 連鎖（パターン条件は PatternBuilder で構築）。

想定 API（MVP）
- if_stmt(cond_json, then_stmts_json[], else_stmts_json[]|null) -> If ノード文字列
- if_expr(cond_json, then_expr_json, else_expr_json, res_name) -> statements_json[]
  - [ Local(res), If(assign res in both branches) ] を返す
- match_stmt(scrut_json, arms[]) -> statements_json[]
- match_expr(scrut_json, arms[], res_name) -> statements_json[]
  - arms: [{ cond_json, guard_json|null, body_expr_json }]
  - cond_json は PatternBuilder で組み立てる（OR/型チェック等）

使用例（式 If）
```
local JB = include "apps/lib/json_builder.nyash"
local CF = include "apps/lib/cf_builder.nyash"   // 実装後に利用

local cond = JB.binary("<", JB.variable("a"), JB.variable("b"))
local then_e = JB.literal_int(10)
local else_e = JB.literal_int(20)
local stmts = CF.if_expr(cond, then_e, else_e, "__res0")
// stmts を元の位置にスプライス（Program 配列へ結合）
```

使用例（式 Match）
```
local JB = include "apps/lib/json_builder.nyash"
local CF = include "apps/lib/cf_builder.nyash"
local PT = include "apps/lib/pattern_builder.nyash"

local scrut = JB.variable("x")
local c_small = PT.or_([ PT.eq(JB.literal_int(0)), PT.eq(JB.literal_int(1)) ])
local arms = [
  { cond_json: c_small, guard_json: JB.variable("small"), body_expr_json: JB.literal_string("small") },
  { cond_json: PT.type_is("IntegerBox", scrut), guard_json: null,
    body_expr_json: /* toString(as_int(scrut)) を別途構築 */ JB.variable("n_to_string") },
  { cond_json: PT.default(), guard_json: null, body_expr_json: JB.literal_string("other") },
]
local stmts = CF.match_expr(scrut, arms, "__res1")
```

備考
- gensym は MacroCtx から受け取る想定（CF 内では与えられた res_name/scrut_name を尊重）。
- 実装は JsonBuilder を用いてノード文字列を生成。
- 将来: cond/guard の短絡は既存の BinaryOp(&&,||) へ委譲し、PyVM/LLVM の規約に従う。

関連
- docs/guides/pattern-builder.md
- docs/guides/if-match-normalize.md
- docs/guides/loopform.md

