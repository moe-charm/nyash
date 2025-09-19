# PatternBuilder — パターン条件ビルダー（コンパイル時メタ）

目的
- Match/If で用いるパターン条件を安全に構築するビルダー。
- 生成物は AST JSON v0 の条件式文字列（BinaryOp/TypeOp 等）で、ControlFlowBuilder に渡して If 連鎖へ合成する。

想定 API（MVP）
- eq(expr_json)                       … scrut == expr
- or_(conds_json_array)               … c1 || c2 || …
- and_(conds_json_array)              … g1 && g2 && …
- type_is(type_name, scrut_json)      … type_check(scrut, type_name)
- default()                           … デフォルト用マーカー（CF側で末尾へ）

使用例
```
local JB = include "apps/lib/json_builder.nyash"
local PT = include "apps/lib/pattern_builder.nyash"

local scrut = JB.variable("x")
local p = PT.or_([ PT.eq(JB.literal_int(0)), PT.eq(JB.literal_int(1)) ])
// guard 付き条件: (p) && small
local cond = PT.and_([ p, JB.variable("small") ])
```

注意
- default() は条件式ではないため、ControlFlowBuilder 側で最後の else に落とす処理を行う。
- 型チェックは TypeOp(check) の JSON を生成するか、既存の JSON 片を組み立てる（実装詳細に依存）。

関連
- docs/guides/controlflow-builder.md
- docs/guides/if-match-normalize.md

