# MIR JSON v0 — Minimal Spec (Gate B)

Status: Active (Phase 20.5 / Gate B). This document fixes the minimal, canonical MIR JSON emitted by the Hakorune Builder and consumed by both VM and LLVM lines.

Root
- Object with `kind:"MIR"`, `version:0`, and `functions:[Fn]`.

Function
- `{ name: string, params: [], blocks: [Block] }`
- For Gate B the parameter list is empty (const/binop/compare examples).

Block
- `{ id: I64, instructions: [Inst] }`
- All blocks must end with a terminator (`ret`/`jump`/`branch`).

I64 Wrapper
- Integers appearing as register ids or integer constants are wrapped as `{type:"i64", value:N}` for clarity and stability.
- Builder（selfhost）では内部的に MapBox/ArrayBox で本スキーマを構築し、数値はこのラッパー形で表現する。

Instructions (P1 minimum)
- `const`  → `{ op:"const", dst:I64, value:I64 }`
- `ret`    → `{ op:"ret",   value:I64 }`
- `binop`  → `{ op:"binop", op_kind:"Add|Sub|Mul|Div|Mod", lhs:I64, rhs:I64, dst:I64 }`
- `compare`→ `{ op:"compare", cmp:"Eq|Ne|Lt|Le|Gt|Ge", lhs:I64, rhs:I64, dst:I64 }`
- `jump`   → `{ op:"jump", target:I64 }`
- `branch` → `{ op:"branch", cond:I64, then:I64, "else":I64 }`

Canonicalization policy
- Object keys are emitted in lexicographic order per object to stabilize output.
- Array order is semantic and preserved as given.
- Gate C（VM直実行）では、以下のフィールドで `{type:"i64", value:N}` を整数 `N` に正規化してから読み込む：
  - `dst`, `ret.value`, `lhs`, `rhs`, `cond`, `then`, `else`, `target`
  - 将来の拡張（call系）では `receiver`/`args[*]` も対象（v0の範囲外）

Notes
- `phi` nodes are not generated in Gate B; merges are expressed as simple constants on each predecessor and a `ret` in the merge block.
- `mir_call` family (Extern/Global/Method/Constructor) remains outside of Gate B scope; basic builders are defined for future gates but not required here.
 - Emitter representation note: 本ドキュメントはシリアライズされた JSON 仕様を定義する。実装内部の Box 形（MapBox/ArrayBox）はこれを忠実に表現するが、実行器（Gate C）側でのカノニカル化により `{type,value}` を整数へアンラップする場合がある。

Examples

const → ret
```
{"version":0,"kind":"MIR","functions":[{"name":"main","blocks":[{"id":{"type":"i64","value":0},"instructions":[{"op":"const","dst":{"type":"i64","value":1},"value":{"type":"i64","value":42}},{"op":"ret","value":{"type":"i64","value":1}}]}]}]}
```

binop(Add)
```
{"version":0,"kind":"MIR","functions":[{"name":"main","blocks":[{"id":{"type":"i64","value":0},"instructions":[{"op":"const","dst":{"type":"i64","value":1},"value":{"type":"i64","value":2}},{"op":"const","dst":{"type":"i64","value":2},"value":{"type":"i64","value":3}},{"op":"binop","op_kind":"Add","lhs":{"type":"i64","value":1},"rhs":{"type":"i64","value":2},"dst":{"type":"i64","value":3}},{"op":"ret","value":{"type":"i64","value":3}}]}]}]}
```

compare(Lt) diamond
```
{"version":0,"kind":"MIR","functions":[{"name":"main","blocks":[
  {"id":{"type":"i64","value":0},"instructions":[
    {"op":"const","dst":{"type":"i64","value":1},"value":{"type":"i64","value":1}},
    {"op":"const","dst":{"type":"i64","value":2},"value":{"type":"i64","value":2}},
    {"op":"compare","cmp":"Lt","lhs":{"type":"i64","value":1},"rhs":{"type":"i64","value":2},"dst":{"type":"i64","value":3}},
    {"op":"branch","cond":{"type":"i64","value":3},"then":{"type":"i64","value":1},"else":{"type":"i64","value":2}}
  ]},
  {"id":{"type":"i64","value":1},"instructions":[
    {"op":"const","dst":{"type":"i64","value":6},"value":{"type":"i64","value":1}},
    {"op":"jump","target":{"type":"i64","value":3}}
  ]},
  {"id":{"type":"i64","value":2},"instructions":[
    {"op":"const","dst":{"type":"i64","value":6},"value":{"type":"i64","value":0}},
    {"op":"jump","target":{"type":"i64","value":3}}
  ]},
  {"id":{"type":"i64","value":3},"instructions":[
    {"op":"ret","value":{"type":"i64","value":6}}
  ]}
]}]}
```
